use std::io;
use std::error::Error;
use std::fs::read_dir;

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

use crate::player::{MetadataType, Player, PlayerCommand};
use crate::expand_tilde;
use crate::song::Song;

enum DisplayMode {
    Queue,
    CurrentTrack,
    FileSelection,
}

pub struct App {
    exit: bool,
    queued_command : Option<PlayerCommand>,

    display_mode : DisplayMode,

    player : Player,

    album_art : Option<StatefulProtocol>,
}

impl App {
    pub fn new() -> Self {
        let album_art_image = App::load_album_cover();

        Self {
            exit : false,
            queued_command : None,

            display_mode : DisplayMode::Queue,

            player : Player::new(),

            album_art : album_art_image,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
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
                            self.player.add_to_queue(song);
                        }
                    }
                } 
            }
        }
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
            KeyCode::Char('i') => self.display_mode = DisplayMode::FileSelection,
            _ => { self.queued_command = None},
        }
    }

    fn load_album_cover() -> Option<StatefulProtocol> {
        let picker = Picker::from_query_stdio().unwrap();

        // Load an image with the image crate.
        let dyn_img = image::ImageReader::open(expand_tilde("~/Music/cover.jpg"));//?.decode()?.resize(1800, 1800, FilterType::Gaussian);

        match dyn_img {
            Ok(img) => {
                let image = picker.new_resize_protocol(img.decode().unwrap().resize(600, 600, FilterType::Gaussian));
                return Some(image);
            }
            _ => return None,
        }
        // Create the Protocol which will be used by the widget.
        

        
    }

    fn exit(&mut self) {
        self.exit = true;
    }
    
}

impl<'a> Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        
        let volume : u8 = (self.player.volume()*100.0).round() as u8;
        let song_title: String = self.player.get_metadata(MetadataType::Title);
        let song_album: String = self.player.get_metadata(MetadataType::Album);
        let song_artist: String = self.player.get_metadata(MetadataType::TrackArtist);
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

        let mode_instructions = Line::from(vec![
            " File Selection ".into(),
            "<i>".blue().bold(),
            " Queue View ".into(),
            "<o>".blue().bold(),
            " Now Playing View ".into(),
            "<p> ".blue().bold(),
        ]);

        let np_block = Block::bordered()
            .title(np_title.left_aligned())
            .title_bottom(mode_instructions.centered())
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
                let trck_title = Line::from(" Next up: ".bold());

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
                        Constraint::Percentage(55),
                        Constraint::Percentage(45),
                    ])
                    .split(layout[1]);

                let curr_trck_dis_title = Line::from(" Current track: ".bold());

                let album_art_title = Line::from(" Cover art ");

                let image = StatefulImage::<StatefulProtocol>::default();

                let album_art_block = Block::bordered()
                    .title_bottom(album_art_title.centered())
                    .border_set(border::THICK);

                match &self.album_art {
                    Some(_) => {
                        let album_art_inner_area = album_art_block.inner(current_layout[1]);

                        album_art_block.render(current_layout[1], buf);

                        image.render(album_art_inner_area, buf, &mut self.album_art.as_mut().unwrap());
                    },
                    None => {
                        Paragraph::new(Line::from("No cover art."))
                            .centered()
                            .block(album_art_block)
                            .render(current_layout[1], buf);
                    }
                }
                

                let track_info_title = Line::from(" Track info ");

                let mut track_info_lines: Vec<Line<'_>> = Vec::new();
                
                // LINES IN TRACK INFO

                let name_span = Line::from(vec![
                        Span::raw(format!("Title: {}", song_title))
                    ]);
                track_info_lines.push(name_span);

                let blank_line = Line::from(vec![
                        Span::raw(" ")
                    ]);
                track_info_lines.push(blank_line);

                let artist_span = Line::from(vec![
                        Span::raw(format!("Artist: {}", song_artist))
                    ]);
                track_info_lines.push(artist_span);

                let blank_line = Line::from(vec![
                        Span::raw(" ")
                    ]);
                track_info_lines.push(blank_line);

                let album_span = Line::from(vec![
                        Span::raw(format!("Album: {}", song_album))
                    ]);
                track_info_lines.push(album_span);


                // END OF LINES IN TRACK INFO

                let track_info_block = Block::bordered()
                    .title(curr_trck_dis_title.left_aligned())
                    .title_bottom(track_info_title.centered())
                    .border_set(border::THICK);

                Paragraph::new(track_info_lines)
                    .centered()
                    .block(track_info_block)
                    .render(current_layout[0], buf);

                
        
            },

            DisplayMode::FileSelection => {
                let fs_title = Line::from(" File Selection: ");

                let fs_instructions = Line::from(vec![
                    " Queue View ".into(),
                    "<o>".blue().bold(),
                    " Now Playing View ".into(),
                    "<p> ".blue().bold(),
                ]);

                let mut fs_lines: Vec<Line<'_>> = Vec::new();


                for n in self.player.player_index..queue_len {
                    let song = self.player.queue().get(n).unwrap();
                    let span = Line::from(vec![
                        Span::raw(format!("  {}", song.title_clone()))
                    ]);
                    fs_lines.push(span);
                }


                let fs_block = Block::bordered()
                    .title(fs_title.left_aligned())
                    .title_bottom(fs_instructions.centered())
                    .border_set(border::THICK);

                Paragraph::new(fs_lines)
                    .left_aligned()
                    .block(fs_block)
                    .render(layout[1], buf);
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
