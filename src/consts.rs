pub const CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";
pub const PLAYLIST_URL: &str = "https://usher.ttvnw.net";

pub mod graphql {
    pub const API_URL: &str = "https://gql.twitch.tv/gql";
    pub const OPERATION_NAME: &str = "PlaybackAccessToken_Template";
    pub const QUERY: &str = r#"
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
}
