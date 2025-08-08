use std::env;

use std::path::PathBuf;

use std::fs::read_dir;

use color_eyre::Result;

// Modules
mod song;
mod player;
mod app;
mod files;

use crate::song::Song;
use crate::player::Player;

use crate::app::{App};

fn main() -> Result<()> {

    color_eyre::install()?;
    
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
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