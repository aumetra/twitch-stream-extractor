#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use {
    ::async_trait::async_trait, error::Result, hls_m3u8::MasterPlaylist,
    serde::de::DeserializeOwned, util::RequestType,
};

#[cfg_attr(feature = "not-send", async_trait(?Send))]
#[cfg_attr(not(feature = "not-send"), async_trait)]
pub trait AsyncClient {
    type Error: Into<Error>;

    async fn get(&self, url: &str) -> Result<String, Self::Error>;
    async fn post_json<T: DeserializeOwned>(
        &self,
        url: &str,
        header: (&str, &str),
        body: String,
    ) -> Result<T, Self::Error>;
}

#[cfg(feature = "reqwest")]
#[cfg_attr(feature = "not-send", async_trait(?Send))]
#[cfg_attr(not(feature = "not-send"), async_trait)]
impl AsyncClient for reqwest::Client {
    type Error = reqwest::Error;

    async fn get(&self, url: &str) -> Result<String, Self::Error> {
        self.get(url).send().await?.text().await
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        url: &str,
        (key, value): (&str, &str),
        body: String,
    ) -> Result<T, Self::Error> {
        self.post(url)
            .header(key, value)
            .body(body)
            .send()
            .await?
            .json::<T>()
            .await
    }
}

#[cfg(feature = "surf")]
#[cfg_attr(feature = "not-send", async_trait(?Send))]
#[cfg_attr(not(feature = "not-send"), async_trait)]
impl AsyncClient for surf::Client {
    type Error = surf::Error;

    async fn get(&self, url: &str) -> Result<String, Self::Error> {
        self.get(url).recv_string().await
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        url: &str,
        (key, value): (&str, &str),
        body: String,
    ) -> Result<T, Self::Error> {
        self.post(url)
            .header(key, value)
            .body(body)
            .recv_json::<T>()
            .await
    }
}

pub struct Extractor<T: AsyncClient> {
    client: T,
}

#[cfg(feature = "reqwest")]
impl Extractor<reqwest::Client> {
    pub fn reqwest() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[cfg(feature = "surf")]
impl Extractor<surf::Client> {
    pub fn surf() -> Self {
        Self {
            client: surf::client(),
        }
    }
}

impl<T: AsyncClient> Extractor<T> {
    pub fn custom(client: T) -> Self {
        Self { client }
    }

    pub async fn stream(&self, channel_name: &str) -> Result<MasterPlaylist<'static>> {
        util::fetch_playlist(&self.client, RequestType::Stream, channel_name).await
    }

    pub async fn vod(&self, vod_id: &str) -> Result<MasterPlaylist<'static>> {
        util::fetch_playlist(&self.client, RequestType::Vod, vod_id).await
    }
}

mod consts;
mod entities;
mod error;
mod util;

pub use {::async_trait, error::Error, hls_m3u8};
