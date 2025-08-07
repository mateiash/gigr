use std::io;
use std::error::Error;

use color_eyre::Result;

use ratatui::{DefaultTerminal, Frame, style::Stylize, symbols::border};
use ratatui::widgets::{Widget, Block, Paragraph};
use ratatui::prelude::{Rect, Buffer, Line, Text, Layout, Direction, Constraint, StatefulWidget};
use ratatui::text::{Span};

use ratatui_image::{picker::Picker, StatefulImage, protocol::StatefulProtocol};

use image;
use image::imageops::FilterType;

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::player::{Player, PlayerCommand};

enum DisplayMode {
    Queue,
    CurrentTrack,
}

pub struct App<'a> {
    exit: bool,
    queued_command : Option<PlayerCommand>,

    display_mode : DisplayMode,

    player : &'a mut Player,

    album_art : StatefulProtocol,
}

impl<'a> App<'a> {
    pub fn new(player : &'a mut Player) -> Self {
        let album_art_image = App::load_album_cover().unwrap();

        Self {
            exit : false,
            queued_command : None,

            display_mode : DisplayMode::Queue,

            player : player,

            album_art : album_art_image,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            match &self.queued_command {
                Some(command) => {
                    match command {
                        PlayerCommand::Prev => {self.player.return_last_song();},
                        PlayerCommand::Skip => {self.player.skip_current_song();},
                        PlayerCommand::VolumeUp => {self.player.change_volume(0.05);},
                        PlayerCommand::VolumeDown => {self.player.change_volume(-0.05);},
                        PlayerCommand::PlayPause => {self.player.play_pause();},
                    }
                },
                None => {},
            }

            self.queued_command = None;
            self.player.update();

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
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
            // PLAYER EVENTS
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('h') => self.queued_command = Some(PlayerCommand::Prev),
            KeyCode::Char('j') => self.queued_command = Some(PlayerCommand::VolumeDown),
            KeyCode::Char('k') => self.queued_command = Some(PlayerCommand::VolumeUp),
            KeyCode::Char('l') => self.queued_command = Some(PlayerCommand::Skip),
            KeyCode::Char(' ') => self.queued_command = Some(PlayerCommand::PlayPause),

            // UI
            KeyCode::Char('p') => self.display_mode = DisplayMode::CurrentTrack,
            KeyCode::Char('o') => self.display_mode = DisplayMode::Queue,
            _ => { self.queued_command = None},
        }
    }

    fn load_album_cover() -> Result<StatefulProtocol, Box<dyn Error>> {
        let picker = Picker::from_query_stdio()?;

        // Load an image with the image crate.
        let dyn_img = image::ImageReader::open("/home/david/Documents/Projects/gigr/target/examples/cover.jpg")?.decode()?.resize(600, 600, FilterType::Gaussian);

        // Create the Protocol which will be used by the widget.
        let image = picker.new_resize_protocol(dyn_img);

        Ok(image)
    }

    fn exit(&mut self) {
        self.exit = true;
    }
    
}

impl<'a> Widget for &mut App<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        
        let volume : u8 = (self.player.volume()*100.0).round() as u8;
        let song_title: String = self.player.current_song_title();
        let playing : bool = self.player.playing();
        let queue_len : usize = self.player.queue().len();


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
            Span::raw(format!("   {}", song_title)),
        ])]);


        Paragraph::new(np_counter_text)
            .left_aligned()
            .block(np_block)
            .render(layout[0], buf);
    
        match self.display_mode {
            DisplayMode::Queue => {
                let trck_title = Line::from(" Upcoming tracks: ".bold());

                let mut track_lines: Vec<Line<'_>> = Vec::new();


                for n in self.player.player_index..queue_len {
                    let song = self.player.queue().get(n).unwrap();
                    let span = Line::from(vec![
                        Span::raw(format!("  {}", song.title_clone()))
                    ]);
                    track_lines.push(span);
                }

                let trck_block = Block::bordered()
                    .title(trck_title.left_aligned())
                    //.title_bottom(instructions.centered())
                    .border_set(border::THICK);
                
                Paragraph::new(track_lines)
                    .left_aligned()
                    .block(trck_block)
                    .render(layout[1], buf);
                    },
            
            DisplayMode::CurrentTrack => {
                let current_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ])
                    .split(layout[1]);

                let curr_trck_dis_title = Line::from(" Current track: ".bold());

                let album_art_title = Line::from(" Cover art ");

                let image = StatefulImage::<StatefulProtocol>::default();

                let album_art_block = Block::bordered()
                    .title(curr_trck_dis_title.left_aligned())
                    .title_bottom(album_art_title.centered())
                    .border_set(border::THICK);

                let album_art_inner_area = album_art_block.inner(current_layout[0]);

                album_art_block.render(current_layout[0], buf);

                image.render(album_art_inner_area, buf, &mut self.album_art);

                let track_info_title = Line::from(" Track info ");

                let mut track_info_lines: Vec<Line<'_>> = Vec::new();
                
                // LINES IN TRACK INFO

                let name_span = Line::from(vec![
                        Span::raw(format!("Title: {}", song_title))
                    ]);
                track_info_lines.push(name_span);

                // END OF LINES IN TRACK INFO

                let track_info_block = Block::bordered()
                    //.title(trck_title.left_aligned())
                    .title_bottom(track_info_title.centered())
                    .border_set(border::THICK);

                Paragraph::new(track_info_lines)
                    .centered()
                    .block(track_info_block)
                    .render(current_layout[1], buf);

                
        
            },
            
        }

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
            match playing {
                true => {"Playing".to_string()}
                false => {"Paused ".to_string()}
            }, volume))
        ])]
            //self.counter.to_string().yellow(),
        );
        
        Paragraph::new(ctrl_counter_text)
            .centered()
            .block(ctrl_block)
            .render(layout[2], buf);
        

    }
}