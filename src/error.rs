use std::error::Error as StdError;

type GeneralError = Box<dyn StdError + Send + Sync + 'static>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error occurred: {0}")]
    Error(#[from] GeneralError),

    #[error("hls_m3u8 failed")]
    HlsM3u8(#[from] hls_m3u8::Error),

    #[error("Missing access token")]
    MissingAccessToken,

    #[cfg(feature = "reqwest")]
    #[error("reqwest failed")]
    Reqwest(#[from] reqwest::Error),

    #[error("serde_json failed")]
    SerdeJson(#[from] serde_json::Error),

    #[cfg(feature = "surf")]
    #[error("surf failed")]
    Surf(surf::Error),
}

#[cfg(feature = "surf")]
impl From<surf::Error> for Error {
    fn from(err: surf::Error) -> Error {
        Error::Surf(err)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
