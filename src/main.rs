use std::env;

use std::io;

use std::path::PathBuf;

use std::fs::read_dir;

use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};

// Modules
mod song;
mod player;
mod app;

use crate::song::Song;
use crate::player::Player;

use crate::app::{App};

fn main() -> Result<()> {

    color_eyre::install()?;
    // ADDING SONGS

    let mut player = Player::new();

    let path = expand_tilde("~/Music");

    if path.is_dir() {

        let mut entries: Vec<_> = read_dir(path)
            .unwrap()
            .map(|res| res.unwrap())
            .collect();

        entries.sort_by_key(|dir| dir.path());


        for entry in entries {
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "mp3" || ext == "flac" || ext == "wav" {
                        let song = Song::new(&path.to_str().unwrap());
                        player.add_to_queue(song);
                    }
                }
            } 
        }
    }

    // EVERYTHING UI
    
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal, &mut player);
    ratatui::restore();
    app_result



}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = env::var_os("HOME") {
            let mut p = PathBuf::from(home);
            p.push(&path[2..]);
            return p;
        }
    }
    PathBuf::from(path)
}