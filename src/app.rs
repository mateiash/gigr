use std::io;

use color_eyre::Result;

use ratatui::{DefaultTerminal, Frame, style::Stylize, symbols::border};
use ratatui::widgets::{Widget, Block, Paragraph};
use ratatui::prelude::{Rect, Buffer, Line, Text};

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

use crate::player::{Player, PlayerCommand};

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    queued_command : Option<PlayerCommand>,
}

impl App {

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal, player : &mut Player) -> Result<()> {
        while !self.exit {
            match &self.queued_command {
                Some(command) => {
                    match command {
                        PlayerCommand::Prev => {player.return_last_song();},
                        PlayerCommand::Skip => {player.skip_current_song();},
                        PlayerCommand::VolumeUp => {player.change_volume(0.05);},
                        PlayerCommand::VolumeDown => {player.change_volume(-0.05);},
                        PlayerCommand::PlayPause => {player.play_pause();},
                    }
                },
                None => {},
            }
            self.queued_command = None;
            player.update();
            
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {

        if crossterm::event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key);
            }
        }
        
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('h') => self.queued_command = Some(PlayerCommand::Prev),
            KeyCode::Char('j') => self.queued_command = Some(PlayerCommand::VolumeDown),
            KeyCode::Char('k') => self.queued_command = Some(PlayerCommand::VolumeUp),
            KeyCode::Char('l') => self.queued_command = Some(PlayerCommand::Skip),
            KeyCode::Char(' ') => self.queued_command = Some(PlayerCommand::PlayPause),
            _ => { self.queued_command = None},
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
    
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" gigr ".bold());
        let instructions = Line::from(vec![
            " Prev ".into(),
            "<h>".blue().bold(),
            " Next ".into(),
            "<l>".blue().bold(),
            " Play / Pause ".into(),
            "<Space> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            //self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}