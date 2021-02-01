use twitch_stream_extractor::{hls_m3u8::tags::VariantStream, Extractor};

#[async_std::main]
async fn main() {
    let extractor = Extractor::surf();
    let vod_playlist = extractor.vod("562766638").await.unwrap();

    for stream in vod_playlist.video_streams() {
        if let VariantStream::ExtXStreamInf { uri, .. } = stream {
            println!("Quality: {}", stream.video().unwrap());
            println!("URL: {}", uri);
            println!("=====================================");
        }
    }
}
