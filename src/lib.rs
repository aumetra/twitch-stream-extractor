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

const API_BASE: &str = "https://api.twitch.tv/api";
const PLAYLIST_DOMAIN: &str = "https://usher.ttvnw.net";

pub type GeneralError = Box<dyn StdError + Sync + Send + 'static>;

/// Trait to define own clients
#[async_trait]
pub trait AsyncClient {
    /// The error returned by the functions
    type Error: Into<Error>;

    /// Execute a GET request
    async fn get(&self, url: &str) -> Result<String, Self::Error>;
    /// Execute a GET request and decode the answer as JSON
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, Self::Error>;
}

#[cfg(feature = "reqwest")]
#[async_trait]
impl AsyncClient for reqwest::Client {
    type Error = reqwest::Error;

    async fn get(&self, url: &str) -> Result<String, Self::Error> {
        self.get(url).send().await?.text().await
    }

    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, Self::Error> {
        self.get(url).send().await?.json::<T>().await
    }
}

#[cfg(feature = "surf")]
#[async_trait]
impl AsyncClient for surf::Client {
    type Error = surf::Error;

    async fn get(&self, url: &str) -> Result<String, Self::Error> {
        self.get(url).recv_string().await
    }

    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, Self::Error> {
        self.get_json(url).await
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

    #[cfg(feature = "surf")]
    #[error("surf error occurred")]
    Surf(String),

    #[error("An error occurred")]
    Error(#[from] GeneralError),
}

#[cfg(feature = "surf")]
impl From<surf::Error> for Error {
    fn from(err: surf::Error) -> Self {
        Self::Surf(err.to_string())
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

enum RequestType {
    Stream,
    Vod,
}

impl RequestType {
    fn as_str(&self) -> &'static str {
        match self {
            RequestType::Stream => "channels",
            RequestType::Vod => "vods",
        }
    }

    fn access_token_url(&self, id: &str) -> String {
        format!(
            "{}/{}/{}/access_token?client_id={}",
            API_BASE,
            self.as_str(),
            id,
            CLIENT_ID
        )
    }

    fn playlist_url(&self, id: &str, access_token: &AccessToken) -> String {
        let query = format!(
            "client_id={}&token={}&sig={}&allow_source&allow_audio_only",
            CLIENT_ID, access_token.token, access_token.signature
        );

        match self {
            RequestType::Stream => {
                format!("{}/api/channel/hls/{}.m3u8?{}", PLAYLIST_DOMAIN, id, query)
            }
            RequestType::Vod => format!("{}/vod/{}.m3u8?{}", PLAYLIST_DOMAIN, id, query),
        }
    }
}

#[derive(Deserialize)]
struct AccessToken {
    token: String,

    #[serde(rename = "sig")]
    signature: String,
}

async fn fetch_playlist<T: AsyncClient>(
    client: &T,
    id: &str,
    request_type: RequestType,
) -> Result<MasterPlaylist<'static>, Error> {
    let access_token_url = request_type.access_token_url(id);
    let access_token = client
        .get_json::<AccessToken>(access_token_url.as_str())
        .await
        .map_err(Into::into)?;

    let playlist_url = request_type.playlist_url(id, &access_token);
    let playlist_data = client
        .get(playlist_url.as_str())
        .await
        .map_err(Into::into)?;

    MasterPlaylist::try_from(playlist_data.as_str())
        .map(MasterPlaylist::into_owned)
        .map_err(Error::from)
}

pub use hls_m3u8;
