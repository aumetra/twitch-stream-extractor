use {
    crate::{AccessToken, AsyncClient, Error, RequestType, CLIENT_ID},
    serde::{Deserialize, Serialize},
};

const GRAPHQL_API_URL: &str = "https://gql.twitch.tv/gql";
const OPERATION_NAME: &str = "PlaybackAccessToken_Template";
const GRAPHQL_QUERY: &str = r#"
query PlaybackAccessToken_Template(
    $login: String!,
    $isLive: Boolean!,
    $vodID: ID!,
    $isVod: Boolean!,
    $playerType: String!
) {
    streamPlaybackAccessToken(
        channelName: $login,
        params: {
            platform: "web",
            playerBackend: "mediaplayer",
            playerType: $playerType
        }
    )
    @include(if: $isLive) {
        value signature __typename
    }
    videoPlaybackAccessToken(
        id: $vodID,
        params: {
            platform: "web",
            playerBackend: "mediaplayer",
            playerType: $playerType
        }
    )
    @include(if: $isVod) {
        value signature __typename
    }
}"#;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlResponseData {
    stream_playback_access_token: Option<AccessToken>,
    video_playback_access_token: Option<AccessToken>,
}

impl GqlResponseData {
    fn access_token(self) -> Result<AccessToken, Error> {
        self.stream_playback_access_token
            .or(self.video_playback_access_token)
            .ok_or(Error::MissingAccessToken)
    }
}

#[derive(Deserialize)]
struct GqlResponse {
    data: GqlResponseData,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccessTokenBodyVars {
    is_live: bool,
    is_vod: bool,
    login: String,

    #[serde(rename = "vodID")]
    vod_id: String,

    player_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccessTokenBody {
    operation_name: String,
    query: String,
    variables: AccessTokenBodyVars,
}

pub(crate) async fn get_access_token<T: AsyncClient>(
    request_type: RequestType,
    id: &str,
    client: &T,
) -> Result<AccessToken, Error> {
    let is_live = request_type == RequestType::Stream;
    let is_vod = !is_live;

    let (login, vod_id) = if is_live { (id, "") } else { ("", id) };

    let body = AccessTokenBody {
        operation_name: OPERATION_NAME.into(),
        query: GRAPHQL_QUERY.into(),
        variables: AccessTokenBodyVars {
            is_live,
            is_vod,
            login: login.into(),
            vod_id: vod_id.into(),
            player_type: "site".into(),
        },
    };

    let access_token_body = serde_json::to_string(&body)?;

    let gql_response = client
        .post_json::<GqlResponse>(GRAPHQL_API_URL, ("client-id", CLIENT_ID), access_token_body)
        .await
        .map_err(Into::into)?;

    gql_response.data.access_token()
}
