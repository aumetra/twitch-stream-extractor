#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::pub_enum_variant_names
)]

//!
//! Extract URLs of live streams or VoD M3U8 playlists from Twitch
//!

use {
    async_trait::async_trait,
    hls_m3u8::{Error as HlsM3u8Error, MasterPlaylist},
    serde::{de::DeserializeOwned, Deserialize},
    std::{boxed::Box, convert::TryFrom, error::Error as StdError},
    thiserror::Error as DeriveError,
};

const CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

const PLAYLIST_DOMAIN: &str = "https://usher.ttvnw.net";

pub type GeneralError = Box<dyn StdError + Sync + Send + 'static>;

/// Trait to define own clients
#[async_trait]
pub trait AsyncClient {
    /// The error returned by the functions
    type Error: Into<Error>;

    /// Execute a GET request
    async fn get(&self, url: &str) -> Result<String, Self::Error>;
    /// Execute a POST request and decode the answer as JSON
    async fn post_json<T: DeserializeOwned>(
        &self,
        url: &str,
        header: &[(&str, &str)],
        body: String,
    ) -> Result<T, Self::Error>;
}

#[cfg(feature = "reqwest")]
#[async_trait]
impl AsyncClient for reqwest::Client {
    type Error = reqwest::Error;

    async fn get(&self, url: &str) -> Result<String, Self::Error> {
        self.get(url).send().await?.text().await
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: String,
    ) -> Result<T, Self::Error> {
        let mut request_builder = self.post(url).body(body);

        for (header_name, header_value) in headers {
            request_builder = request_builder.header(*header_name, *header_value);
        }

        request_builder.send().await?.json::<T>().await
    }
}

#[cfg(feature = "surf")]
#[async_trait]
impl AsyncClient for surf::Client {
    type Error = surf::Error;

    async fn get(&self, url: &str) -> Result<String, Self::Error> {
        self.get(url).recv_string().await
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: String,
    ) -> Result<T, Self::Error> {
        let mut request_builder = self.post(url).body(body);

        for (header_name, header_value) in headers {
            request_builder = request_builder.header(*header_name, *header_value);
        }

        request_builder.recv_json::<T>().await
    }
}

/// Combined error type
#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("hls_m3u8 error occurred")]
    HlsM3u8(#[from] HlsM3u8Error),

    #[cfg(feature = "reqwest")]
    #[error("reqwest error occurred")]
    Reqwest(#[from] reqwest::Error),

    #[error("serde-json error occurred")]
    SerdeJson(#[from] serde_json::Error),

    #[cfg(feature = "surf")]
    #[error("surf error occurred")]
    Surf(surf::Error),

    #[error("An error occurred")]
    Error(#[from] GeneralError),

    #[error("Missing access token")]
    MissingAccessToken,
}

#[cfg(feature = "surf")]
impl From<surf::Error> for Error {
    fn from(err: surf::Error) -> Self {
        Self::Surf(err)
    }
}

/// The URL extractor
pub struct Extractor<T: AsyncClient> {
    client: T,
}

impl<T: AsyncClient> Extractor<T> {
    /// Construct a new extractor using the given client
    pub fn custom(client: T) -> Self {
        Self { client }
    }

    /// Extract the playlist for a live stream
    ///
    /// # Errors
    ///
    /// This can either fail because:
    /// * the access token response is malformed
    /// * internet connectivity issues
    /// * Twitch server issues
    /// * Twitch changed the APIs
    pub async fn stream(&self, channel_name: &'_ str) -> Result<MasterPlaylist<'static>, Error> {
        fetch_playlist(&self.client, channel_name, RequestType::Stream).await
    }

    /// Extract the playlist for a VoD
    ///
    /// # Errors
    ///
    /// This can either fail because:
    /// * the access token response is malformed
    /// * internet connectivity issues
    /// * Twitch server issues
    /// * Twitch changed the APIs
    pub async fn vod(&self, vod_id: &'_ str) -> Result<MasterPlaylist<'static>, Error> {
        fetch_playlist(&self.client, vod_id, RequestType::Vod).await
    }
}

#[cfg(feature = "reqwest")]
impl Extractor<reqwest::Client> {
    /// Create a new extractor using a standard reqwest client
    pub fn reqwest() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[cfg(feature = "surf")]
impl Extractor<surf::Client> {
    /// Create a new extractor using a standard surf client
    pub fn surf() -> Self {
        Self {
            client: surf::client(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum RequestType {
    Stream,
    Vod,
}

impl RequestType {
    fn playlist_url(self, id: &str, access_token: &AccessToken) -> String {
        let query = format!(
            "client_id={}&token={}&sig={}&allow_source&allow_audio_only",
            CLIENT_ID, access_token.value, access_token.signature
        );

        match self {
            RequestType::Stream => {
                format!("{}/api/channel/hls/{}.m3u8?{}", PLAYLIST_DOMAIN, id, query)
            }
            RequestType::Vod => format!("{}/vod/{}.m3u8?{}", PLAYLIST_DOMAIN, id, query),
        }
    }
}

#[derive(Default, Deserialize)]
struct AccessToken {
    value: String,
    signature: String,
}

async fn fetch_playlist<T: AsyncClient>(
    client: &T,
    id: &str,
    request_type: RequestType,
) -> Result<MasterPlaylist<'static>, Error> {
    let access_token = graphql_api::get_access_token(request_type, id, client).await?;

    let playlist_url = request_type.playlist_url(id, &access_token);
    let playlist_data = client
        .get(playlist_url.as_str())
        .await
        .map_err(Into::into)?;

    MasterPlaylist::try_from(playlist_data.as_str())
        .map(MasterPlaylist::into_owned)
        .map_err(Error::from)
}

mod graphql_api;

pub use hls_m3u8;
