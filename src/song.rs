use std::path::{PathBuf};

use lofty::prelude::{ItemKey};
use lofty::probe::{Probe};
use lofty::file::TaggedFileExt;
use lofty::file::AudioFile;
pub struct Song {
    pub file_path : String,
    
    pub title : String,
    pub artist : String,
    pub album : String,
    pub samplerate : usize,
    pub channels : usize,
}

impl Song {
    pub fn new(file_path : &str) -> Self {
        //let buffered = BufReader::new(file);
        let path = std::path::Path::new(&file_path);
        let tagged_file = Probe::open(path).unwrap()
        .read().unwrap();

        if let Some(tag) = tagged_file.primary_tag() {
            let title = if let Some(title) = tag.get_string(&ItemKey::TrackTitle) {
                title.to_string()
            } else {
                String::from("-")
            };

            let artist = if let Some(artist) = tag.get_string(&ItemKey::TrackArtist) {
                artist.to_string()
            } else {
                String::from("-")
            };

            let album= if let Some(album) = tag.get_string(&ItemKey::AlbumTitle) {
                album.to_string()
            } else {
                String::from("-")
            };

            let properties = tagged_file.properties();
            let sample_rate = properties.sample_rate().unwrap();
            let cc = properties.channels().unwrap();

            Self {
                file_path : file_path.to_string(),
                //source : Decoder::try_from(buffered).unwrap(),
                title : title,
                artist : artist,
                album : album,
                samplerate : sample_rate.try_into().unwrap(),
                channels : cc.try_into().unwrap(),

            }
        } else {

            Self {
                file_path : file_path.to_string(),
                //source : Decoder::try_from(buffered).unwrap(),
                title : String::from("-"),
                artist : String::from("-"),
                album : String::from("-"),
                samplerate : 44100,
                channels : 2,

            }
        }
    }

    pub fn title_clone(&self) -> String{
        return self.title.clone();
    }
    pub fn album_clone(&self) -> String{
        return self.album.clone();
    }
    pub fn artist_clone(&self) -> String{
        return self.artist.clone();
    }

    pub fn file_path_clone(&self) -> String{
        return self.file_path.clone();
    }
    pub fn file_path_as_path(&self) -> PathBuf{
        return PathBuf::from(&self.file_path);
    }
}