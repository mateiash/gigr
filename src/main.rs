use std::fs::File;
use rodio::{Decoder, OutputStream, source::Source};

fn main() {

    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");

    let sink = rodio::Sink::connect_new(&stream_handle.mixer());

    let file = File::open("examples/music.flac").unwrap();

    let source = Decoder::try_from(file).unwrap();

    stream_handle.mixer().add(source);

    std::thread::sleep(std::time::Duration::from_secs(5));
}
