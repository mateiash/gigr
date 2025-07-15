use std::collections::VecDeque;
use std::io::BufReader;

use std::fs::File;

use rodio::OutputStream;
use rodio::Sink;
use rodio::Decoder;

use crate::song::Song;

pub struct Player{
    stream_handle : OutputStream,
    sink : Sink,

    queue : VecDeque<Song>,
    current_song : Option<Song>,
}

impl Player {
    pub fn new() -> Self{
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        
        Self {
            sink : sink,
            stream_handle : stream_handle,
            queue : VecDeque::new(),
            current_song : None,
        }
    }

    pub fn add_to_queue(&mut self, song : Song) -> () {
        self.queue.push_back(song);
    }

    /*
    pub fn sleep_until_end(&self) -> () {
        self.sink.sleep_until_end();
    }
    */
    pub fn update(&mut self) -> () {
        if !self.sink.empty() {
            return;
        }

        match self.queue.pop_front() {
            Some(song) => {
                self.current_song = Some(song);

                let song_ref = self.current_song.as_ref().unwrap();
                let file = File::open(song_ref.file_path.clone()).unwrap();
                let buffered = BufReader::new(file);
                let source: Decoder<BufReader<File>> = Decoder::try_from(buffered).unwrap();
                self.sink.append(source);

            }
            None => {
                self.current_song = None;
            }
        }
    }

    pub fn current_song_title(&self) -> String {
        match &self.current_song {
            Some(song) => {
                return song.title.clone();
            },
            None => {
                return "Nothing".to_string();
            },
        }
    }
}