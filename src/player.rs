use std::io::BufReader;

use std::fs::File;

use rodio::OutputStream;
use rodio::Sink;
use rodio::Decoder;

use crate::song::Song;

pub struct Player{
    stream_handle : OutputStream,
    sink : Sink,

    queue : Vec<Song>,
    pub player_index : usize,

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

            queue : Vec::new(),
            player_index : 0,

            current_song : None,

            volume : 1.0,
        }
    }

    pub fn add_to_queue(&mut self, song : Song) -> () {
        self.queue.push(song);
    }

    pub fn update(&mut self) -> bool {
        if !self.sink.empty() {
            return false;
        }

        self.player_index += 1;

        if self.player_index > self.queue.len() {
            //self.player_index = 0;
            self.player_index -= 1;
            return false;
        } 

        let song_ref = self.queue.get(self.player_index - 1).unwrap();

        self.current_song = Some(Song::new(&song_ref.file_path_clone()));

        let file = File::open(song_ref.file_path.clone()).unwrap();
        let buffered = BufReader::new(file);
        let source: Decoder<BufReader<File>> = Decoder::try_from(buffered).unwrap();
        self.sink.append(source);
        return true;
    }

    pub fn get_metadata(&self, metadata_type : MetadataType) -> String {
        match self.sink.empty() {
            false => {
                match &self.current_song {
                    Some(curr_song) => {
                        match metadata_type {
                            MetadataType::Album => return curr_song.album_clone(),
                            MetadataType::Title => return curr_song.title_clone(),
                            MetadataType::TrackArtist => return curr_song.artist_clone(),
                        }
                        
                    
                    },
                    None => {return "Nothing".to_string();},
                }
                
            },
            true => {
                return "Nothing".to_string();
            },
        }
    }

    pub fn skip_current_song(&mut self) -> () {
        if self.sink.empty() {
            return;
        }

        self.sink.skip_one();
    }

    pub fn clear_queue(&mut self) -> () {
        if self.sink.empty() {
            return;
        }

        self.queue.clear();
        self.player_index = 0;
    }

    pub fn return_last_song(&mut self) -> () {
        if self.sink.empty() {
            return;
        }

        match self.player_index {
            0 => {},
            1 => {self.player_index = 0},
            _ => {self.player_index -= 2},
        }
        
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

    pub fn queue(&self) -> &Vec<Song> {
        return &self.queue;
    }

    pub fn current_song(&self) -> Option<&Song> {
        match &self.current_song{
            Some(song) => return Some(song),
            None => return None,
        }
    }

}

#[derive(Debug)]
pub enum PlayerCommand {
    Skip,
    Prev,
    PlayPause,
    VolumeUp,
    VolumeDown,
}

pub enum MetadataType {
    Title,
    TrackArtist,
    Album,
}