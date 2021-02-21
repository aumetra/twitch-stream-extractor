use {
    crate::{
        consts::{
            graphql::{self, OPERATION_NAME, QUERY},
            CLIENT_ID, PLAYLIST_URL,
        },
        entities::{
            graphql::{
                request::{Body, BodyVars},
                Response,
            },
            AccessToken,
        },
        error::Result,
        AsyncClient,
    },
    hls_m3u8::MasterPlaylist,
    std::convert::TryFrom,
};

#[derive(Clone, Copy, PartialEq)]
pub enum RequestType {
    Stream,
    Vod,
}

impl RequestType {
    pub fn playlist_url(
        self,
        id: &str,
        AccessToken {
            ref value,
            ref signature,
        }: &AccessToken,
    ) -> String {
        let query = format!(
            "client_id={}&token={}&sig={}&allow_source&allow_audio_only",
            CLIENT_ID, value, signature
        );

        match self {
            RequestType::Stream => {
                format!("{}/api/channel/hls/{}.m3u8?{}", PLAYLIST_URL, id, query)
            }
            RequestType::Vod => format!("{}/vod/{}.m3u8?{}", PLAYLIST_URL, id, query),
        }
    }
}

async fn fetch_access_token<T: AsyncClient>(
    client: &T,
    request_type: RequestType,
    id: &str,
) -> Result<AccessToken> {
    let is_live = request_type == RequestType::Stream;

    let graphql_body = Body {
        operation_name: OPERATION_NAME.into(),
        query: QUERY.into(),
        variables: BodyVars::new(is_live, id),
    };
    let graphql_body = serde_json::to_string(&graphql_body)?;

    let graphql_response = client
        .post::<Response>(graphql::API_URL, ("client-id", CLIENT_ID), graphql_body)
        .await
        .map_err(Into::into)?;

    graphql_response.access_token()
}

pub async fn fetch_playlist<T: AsyncClient>(
    client: &T,
    request_type: RequestType,
    id: &str,
) -> Result<MasterPlaylist<'static>> {
    let access_token = fetch_access_token(client, request_type, id).await?;

    let playlist_url = request_type.playlist_url(id, &access_token);
    let playlist_data = client
        .get(playlist_url.as_str())
        .await
        .map_err(Into::into)?;

    MasterPlaylist::try_from(playlist_data.as_str())
        .map(MasterPlaylist::into_owned)
        .map_err(Into::into)
}
