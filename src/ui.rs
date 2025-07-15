use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame,
};

use crate::player::Player;

pub fn ui<B: Backend>(f: &mut Frame<B>, player : &Player) {

    let song_title = player.current_song_title();

   let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10)
            ].as_ref()
        )
        .split(f.size());

    let block = Block::default()
         .title(format!("gigr - Now playing: {}", song_title))
         .borders(Borders::ALL);

    f.render_widget(block, chunks[0]);

    let block = Block::default()
         .title("Tracks")
         .borders(Borders::ALL);

    f.render_widget(block, chunks[1]);

}