use std::io;
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

    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let song : Song = Song::new("/home/david/Documents/Projects/gigr/target/examples/music.flac");
    let song2 : Song = Song::new("/home/david/Documents/Projects/gigr/target/examples/music2.flac");
    let mut player : Player = Player::new();
    player.add_to_queue(song);
    player.add_to_queue(song2);

    loop {
        player.update();

        terminal.draw(|f| ui(f, &player))?;
        
        // Basic event handling
        if crossterm::event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
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
