use twitch_stream_extractor::{hls_m3u8::tags::VariantStream, Extractor};

#[async_std::main]
async fn main() {
    let extractor = Extractor::surf();
    let channel_playlist = extractor.stream("sleepy").await.unwrap();

    for stream in channel_playlist.video_streams() {
        if let VariantStream::ExtXStreamInf { uri, .. } = stream {
            println!("Quality: {}", stream.video().unwrap());
            println!("URL: {}", uri);
            println!("=====================================");
        }
    }
}
