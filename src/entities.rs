use serde::Deserialize;

#[derive(Deserialize)]
pub struct AccessToken {
    pub value: String,
    pub signature: String,
}

pub mod graphql {
    use {
        super::AccessToken,
        crate::error::{Error, Result},
        serde::Deserialize,
    };

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResponseData {
        stream_playback_access_token: Option<AccessToken>,
        video_playback_access_token: Option<AccessToken>,
    }

    #[derive(Deserialize)]
    pub struct Response {
        data: ResponseData,
    }

    impl Response {
        pub fn access_token(self) -> Result<AccessToken> {
            self.data
                .stream_playback_access_token
                .or(self.data.video_playback_access_token)
                .ok_or(Error::MissingAccessToken)
        }
    }

    pub mod request {
        use serde::Serialize;

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        pub struct BodyVars {
            pub is_live: bool,
            pub is_vod: bool,

            // Username
            pub login: String,

            #[serde(rename = "vodID")]
            pub vod_id: String,

            // Standard player type: "site"
            pub player_type: String,
        }

        impl BodyVars {
            pub fn new(is_live: bool, id: &str) -> Self {
                Self {
                    is_live,
                    is_vod: !is_live,

                    login: id.into(),
                    vod_id: id.into(),

                    player_type: "site".into(),
                }
            }
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Body {
            pub operation_name: String,
            pub query: String,
            pub variables: BodyVars,
        }
    }
}
