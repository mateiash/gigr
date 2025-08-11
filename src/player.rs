use std::io::BufReader;

use std::fs::File;

use color_eyre::eyre::Error;
use rodio::OutputStream;
use rodio::Sink;
use rodio::Decoder;

use rustfft::{FftPlanner, num_complex::Complex};

use crate::song::Song;
use crate::expand_tilde;

const EQ_BUFFER_SIZE : usize = 512;

pub struct Player{
    stream_handle : OutputStream,
    sink : Sink,

    queue : Vec<Song>,
    pub player_index : usize,

    current_song : Option<Song>,

    volume : f32,

    fft_planner : FftPlanner<f32>,
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

            fft_planner : FftPlanner::new(),
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

    pub fn eq_bands(&mut self, n_bands : i32) -> Option<Vec<f32>>{
        if self.sink.empty() {
            return None;
        }
        
        let song_ref = self.current_song.as_ref().unwrap();

        let start : usize = (self.sink.get_pos().as_millis() as usize) * 
                            song_ref.samplerate * song_ref.channels / 1000;
        
        let file = File::open(expand_tilde(&self.current_song().unwrap().file_path_clone())).unwrap();


        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        let left_channel : Vec<f32> = decoder
        .skip(start)
        .take(EQ_BUFFER_SIZE*2)
        .enumerate()
        .filter_map(|(i, val)| if i % 2 == 0 { Some(val) } else { None })
        .collect();
        /* 
        let left_channel : Vec<f32> = waveform
            .iter()
            .enumerate()
            .filter_map(|(i, &val)| if i % 2 == 0 { Some(val) } else { None })
            .collect();
        */
        let mut buffer: Vec<Complex<f32>> = left_channel
        .iter()
        .map(|&re| Complex { re, im: 0.0 })
        .collect();

        let fft = self.fft_planner.plan_fft_forward(buffer.len());

        fft.process(&mut buffer);


        // You can, for example, get magnitudes like this:
        let magnitudes: Vec<f32> = buffer.iter()
            .map(|c| c.norm())  // norm() gives magnitude of the complex number
            .collect();

        //println!("Magnitudes: {}", magnitudes.len());

            /* 
        let samples = decoder  // unwrap the Result, discard errors
            .map(|sample| Sample::to_f32(&sample))
            .take(FFT_SIZE)
            .collect();

    */  return
            Some(Self::split_into_bands(&magnitudes, 44100.0, EQ_BUFFER_SIZE, n_bands as usize).unwrap());
    }

    fn split_into_bands(
        magnitudes: &[f32],
        sample_rate: f32,
        fft_size: usize,
        n_bands: usize,
    ) -> Option<Vec<f32>> {
        let nyquist = sample_rate / 2.0;

        let freq_per_bin = nyquist / magnitudes.len() as f32;

        let min_freq: f32 = 20.0;
        let max_freq: f32 = nyquist;

        let log_min = min_freq.ln();
        let log_max = max_freq.ln();
        let band_edges: Vec<f32> = (0..=n_bands)
            .map(|i| {
                let t = i as f32 / n_bands as f32;
                (log_min + t * (log_max - log_min)).exp()
            })
            .collect();

        let mut bands = vec![0.0; n_bands];
        let mut counts = vec![0usize; n_bands];

        for (i, &mag) in magnitudes.iter().enumerate() {
            let freq = i as f32 * freq_per_bin;
            if freq < min_freq || freq > max_freq {
                continue;
            }

            if let Some(band) = band_edges
                .windows(2)
                .position(|w| freq >= w[0] && freq < w[1])
            {
                bands[band] += mag;
                counts[band] += 1;
            }
        }

        for (b, &count) in bands.iter_mut().zip(&counts) {
            if count > 0 {
                *b /= count as f32;
            }
        }

        if let Some(&max_val) = bands.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            if max_val > 0.0 {
                for b in &mut bands {
                    *b /= max_val;
                }
            }
        }

        Some(bands)
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