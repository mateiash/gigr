use tui::{
    backend::Backend, layout::{Constraint, Direction, Layout}, style::{Modifier, Style}, widgets::{Block, Borders, Paragraph}, Frame
};

use tui::text::Span;
use tui::text::Spans;

use crate::player::Player;

pub fn ui<B: Backend>(f: &mut Frame<B>, player : &Player) {

    let song_title = player.current_song_title();
    let volume = (player.volume()*100.0).round() as u8;

   let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3)
            ].as_ref()
        )
        .split(f.size());

    let title_bar = Paragraph::new(vec![
            Spans::from(Span::raw(format!("   {}", song_title))),
        ]).block(
    Block::default()
         .title(" gigr - Now playing: ")
         .borders(Borders::ALL));

    f.render_widget(title_bar, chunks[0]);


    // TRACKS

    let mut track_lines = Vec::new();

    for song in player.queue() {
        let span = Span::raw(format!("  {}", song.title_clone()));
        track_lines.push(Spans::from(span));
    }


    let tracks = Paragraph::new(track_lines).block(
    Block::default()
         .title(" Upcoming Tracks: ")
         .borders(Borders::ALL));
    f.render_widget(tracks, chunks[1]);

    let controls = Paragraph::new(vec![
        //Spans::from(Span::raw(format!("  Volume: {}%", volume))),
        Spans::from(Span::styled(format!("  {} - Volume: {}%",
            match player.playing() {
                true => {"Playing".to_string()}
                false => {"Paused ".to_string()}
            }, volume), Style::default().add_modifier(Modifier::BOLD)))
    ]).block(
    Block::default()
         .title(" Controls: ")
         .borders(Borders::ALL));

    f.render_widget(controls, chunks[2]);

}