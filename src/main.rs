use std::env;

use std::io;

use std::path::PathBuf;

use std::fs::read_dir;

use tui::{
    backend::CrosstermBackend,
    Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// Modules
mod song;
mod player;
mod ui;

use crate::song::Song;
use crate::player::Player;
use crate::ui::ui;

fn main() -> Result<(), io::Error> {

    // UI INIT

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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

    // UI LOOP

    loop {
        player.update();

        terminal.draw(|f| ui(f, &player))?;
        
        // Basic event handling
        if crossterm::event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {break},
                    KeyCode::Char('l') => {player.skip_current_song()},
                    KeyCode::Char('j') => {player.change_volume(-0.05)},
                    KeyCode::Char('k') => {player.change_volume(0.05)},
                    KeyCode::Char(' ') => {player.play_pause()},
                    _ => {},
                }
            }
        }

    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    //player.sleep_until_end();

    Ok(())

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