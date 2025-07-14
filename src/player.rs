use std::io::BufReader;

use std::fs::File;

use rodio::OutputStream;
use rodio::Sink;
use rodio::Decoder;

use crate::song::Song;

pub struct Player{
    stream_handle : OutputStream,
    sink : Sink,
    
}

impl Player {
    pub fn new() -> Self{
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        
        Self {
            sink : sink,
            stream_handle : stream_handle,
        }
    }

    pub fn add_to_queue(&self, song : &Song) -> () {
        let file = File::open(song.file_path.clone()).unwrap();
        let buffered = BufReader::new(file);
        let source = Decoder::try_from(buffered).unwrap();
        self.sink.append(source);
    }

    pub fn sleep_until_end(&self) -> () {
        self.sink.sleep_until_end();
    }
}