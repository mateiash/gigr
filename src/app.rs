use std::io;

use color_eyre::Result;

use ratatui::{DefaultTerminal, Frame, style::Stylize, symbols::border};
use ratatui::widgets::{Widget, Block, Paragraph};
use ratatui::prelude::{Rect, Buffer, Line, Text, Layout, Direction, Constraint};
use ratatui::text::{Span};
use ratatui::style::{Style, Modifier};

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

use crate::player::{Player, PlayerCommand};

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    queued_command : Option<PlayerCommand>,

    volume : u8,
    song_title : String,
    playing : bool,
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

            self.volume = (player.volume()*100.0).round() as u8;
            self.song_title = player.current_song_title();
            self.playing = player.playing();

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
        
        let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

        // NOW PLAYING ELEMENT

        let np_title = Line::from(" gigr - Now playing: ".bold());
        let np_block = Block::bordered()
            .title(np_title.left_aligned())
            //.title_bottom(instructions.centered())
            .border_set(border::THICK);

        let np_counter_text = Text::from(vec![Line::from(vec![
            Span::raw(format!("   {}", self.song_title)),
        ])]);


        Paragraph::new(np_counter_text)
            .left_aligned()
            .block(np_block)
            .render(layout[0], buf);
    
        // TRACKS ELEMENT
        
        let trck_title = Line::from(" Upcoming tracks: ".bold());

        let trck_block = Block::bordered()
            .title(trck_title.left_aligned())
            //.title_bottom(instructions.centered())
            .border_set(border::THICK);

        let trck_counter_text = Text::from(vec![Line::from(vec![
            "    -".into(),
            //self.counter.to_string().yellow(),
        ])]);
        
        Paragraph::new(trck_counter_text)
            .left_aligned()
            .block(trck_block)
            .render(layout[1], buf);
        

        // CONTROLS ELEMENT
    
        let ctrl_title = Line::from(" Controls: ".bold());
        let instructions = Line::from(vec![
            " Prev ".into(),
            "<h>".blue().bold(),
            " Vol - ".into(),
            "<j>".blue().bold(),
            " Vol + ".into(),
            "<k>".blue().bold(),
            " Next ".into(),
            "<l>".blue().bold(),
            " Play / Pause ".into(),
            "<Space> ".blue().bold(),
        ]);

        let ctrl_block = Block::bordered()
            .title(ctrl_title.left_aligned())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let ctrl_counter_text = Text::from(vec![Line::from(vec![
        //Spans::from(Span::raw(format!("  Volume: {}%", volume))),
        Span::raw(format!("  {} - Volume: {}%",
            match self.playing {
                true => {"Playing".to_string()}
                false => {"Paused ".to_string()}
            }, self.volume))
        ])]
            //self.counter.to_string().yellow(),
        );
        
        Paragraph::new(ctrl_counter_text)
            .centered()
            .block(ctrl_block)
            .render(layout[2], buf);
        

    }
}