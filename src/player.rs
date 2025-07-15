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

    volume : f32,
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

            volume : 1.0,
        }
    }

    pub fn add_to_queue(&mut self, song : Song) -> () {
        self.queue.push_back(song);
    }

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

    pub fn skip_current_song(&self) -> () {
        self.sink.skip_one();
    }

    fn set_volume(&mut self, volume : f32) -> () {
        if volume > 1.0 {
            self.volume = 1.0;
            self.sink.set_volume(1.0);
        } else if volume < 0.0 {
            self.volume = 0.0;
            self.sink.set_volume(0.0);
        } else {
            self.volume = volume;
            self.sink.set_volume(volume);
        }
    }

    pub fn change_volume(&mut self, amount : f32) -> () {
        self.set_volume(self.volume + amount);
    }

    pub fn volume(&self) -> f32 {
        return self.volume;
    }

    pub fn play_pause(&self) -> () {
        match self.sink.is_paused() {
            true => {self.sink.play();},
            false => {self.sink.pause();},
        }
    }

    pub fn playing(&self) -> bool {
        return !self.sink.is_paused();
    }

}