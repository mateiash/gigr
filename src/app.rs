use std::io;
use std::fs::DirEntry;
use std::path::PathBuf;

use color_eyre::Result;

use ratatui::style::Modifier;
use ratatui::{DefaultTerminal, Frame, style::Stylize, symbols::border};
use ratatui::widgets::{Widget, Block, Paragraph};
use ratatui::prelude::{Rect, Buffer, Line, Text, Layout, Direction, Constraint, StatefulWidget};
use ratatui::text::{Span};

use ratatui_image::{picker::Picker, StatefulImage, protocol::StatefulProtocol};

use image;
use image::imageops::FilterType;

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::files::FileSelector;
use crate::player::{MetadataType, Player, PlayerCommand};
use crate::expand_tilde;
use crate::song::Song;

const EQ_POS_CHAR : char = '■';
const EQ_NEG_CHAR : char = ' ';

#[derive(PartialEq)]
enum DisplayMode {
    Title,
    Queue,
    CurrentTrack,
    FileSelection,
}

pub struct App {
    exit: bool,
    queued_command : Option<PlayerCommand>,

    display_mode : DisplayMode,

    player : Player,
    file_selector : FileSelector,

    files_queue : Option<Vec<DirEntry>>,

    album_art : Option<StatefulProtocol>,
}

impl App {
    pub fn new() -> Self {
        let album_art_image = App::load_album_cover(expand_tilde("~/Music/cover.jpg"));

        Self {
            exit : false,
            queued_command : None,

            display_mode : DisplayMode::Title,

            player : Player::new(),
            file_selector : FileSelector::new(expand_tilde("~/Music")),

            files_queue : None,

            album_art : album_art_image,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            match &self.files_queue {
                Some(entries) => {
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
                },
                None => {},
            }
            self.files_queue = None;

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
            let update : bool = self.player.update();


            if update {
                let mut img_path = self.player.current_song().unwrap().file_path_as_path();
                img_path.pop();
                img_path.push("Cover.jpg");
                self.album_art = App::load_album_cover(
                    img_path
                );
            }

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
            // DISPLAY MODE SELECTION

            KeyCode::Char('p') => self.display_mode = DisplayMode::CurrentTrack,
            KeyCode::Char('o') => self.display_mode = DisplayMode::Queue,
            KeyCode::Char('i') => self.display_mode = DisplayMode::FileSelection,

            _ => { self.queued_command = None},
        }

        // FILE SELECTION
        if self.display_mode == DisplayMode::FileSelection {
            match key_event.code {
                KeyCode::Char('s') => self.file_selector.move_down(),
                KeyCode::Char('d') => self.file_selector.move_up(),
                KeyCode::Char('f') => self.file_selector.move_forwards(),
                KeyCode::Char('a') => self.file_selector.move_back(),
                KeyCode::Enter => self.files_queue = self.file_selector.queue_selection(),

                _ => {},
            }
        }
    }

    fn load_album_cover(path : PathBuf) -> Option<StatefulProtocol> {
        let picker = Picker::from_query_stdio().unwrap();

        // Load an image with the image crate.
        let dyn_img = image::ImageReader::open(path);

        match dyn_img {
            Ok(img) => {
                let image = picker.new_resize_protocol(img.decode().unwrap().resize(600, 600, FilterType::Gaussian));
                return Some(image);
            }
            _ => return None,
        }
        
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
        let playback_time = self.player.playback_time();


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

        let np_playback_time = Text::from(vec![Line::from(vec![
            match playback_time.1 < 10 {
                true => Span::raw(format!("{}:0{}   ", playback_time.0, playback_time.1)),
                false => Span::raw(format!("{}:{}   ", playback_time.0, playback_time.1)),
            }
            
        ])]);

        Paragraph::new(np_counter_text)
            .left_aligned()
            .block(np_block.clone())
            .render(layout[0], buf);

        Paragraph::new(np_playback_time)
            .right_aligned()
            .block(np_block)
            .render(layout[0], buf);

        match self.display_mode {
            DisplayMode::Title => {
                let title_block = Block::bordered()
                    //.title(trck_title.left_aligned())
                    //.title_bottom(instructions.centered())
                    .border_set(border::THICK);

                let mut title_lines: Vec<Line<'_>> = Vec::new();

                let line1 = Line::from(vec![
                        Span::raw("         oo                    ")
                    ]).blue();
                title_lines.push(line1);

                let line2 = Line::from(vec![
                        Span::raw("                               ")
                    ]).blue();
                title_lines.push(line2);

                let line3 = Line::from(vec![
                        Span::raw(".d8888b. dP .d8888b. 88d888b.  ")
                    ]).blue();
                title_lines.push(line3);

                let line4 = Line::from(vec![
                        Span::raw("88'  `88 88 88'  `88 88'  `88  ")
                    ]).blue();
                title_lines.push(line4);

                let line5 = Line::from(vec![
                        Span::raw("88.  .88 88 88.  .88 88        ")
                    ]).blue();
                title_lines.push(line5);

                let line6 = Line::from(vec![
                        Span::raw("`8888P88 dP `8888P88 dP        ")
                    ]).blue();
                title_lines.push(line6);

                let line7 = Line::from(vec![
                        Span::raw("     .88         .88           ")
                    ]).blue();
                title_lines.push(line7);

                let line8 = Line::from(vec![
                        Span::raw(" d8888P      d8888P by mateiash")
                    ]).blue();
                title_lines.push(line8);

                Paragraph::new(title_lines)
                            .centered()
                            .block(title_block)
                            .render(layout[1], buf);
            }
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
                
                let current_layout_info = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        Constraint::Length(7),
                        Constraint::Min(0),
                    ])
                    .split(current_layout[0]);

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
                    .render(current_layout_info[0], buf);

                // EQ

                let track_eq_block = Block::bordered()
                    //.title(curr_trck_dis_title.left_aligned())
                    .title_bottom(Line::from(" EQ ").centered())
                    .border_set(border::THICK);
                
                let width: f32 = (current_layout_info[1].width - 2) as f32;
                let height: f32 = (current_layout_info[1].height - 2) as f32;

                match self.player.eq_bands(width as i32) {
                    Some(bands) => {

                        let mut eq_chars: Vec<Line<'_>> = Vec::new();

                        for _ in 0..2 {
                            let mut line = String::from("");
                            for _ in 0..bands.len() {
                                line.push(EQ_NEG_CHAR);                            
                            }
                            let name_span = Line::from(vec![
                                Span::raw(format!("{}", line)).blue()
                                ]);
                            eq_chars.push(name_span);
                        }

                        for i in 0..height.round() as isize - 3{
                            let mut line = String::from("");
                            for j in 0..bands.len() {
                                let element = *bands.get(j).unwrap();
                                if element > 1f32 - (i+1) as f32 *1f32/height {
                                    line.push(EQ_POS_CHAR);
                                } else { 
                                    line.push(EQ_NEG_CHAR);
                                }
                            }
                            let name_span = Line::from(vec![
                                    Span::raw(format!("{}", line)).blue()
                                ]);
                            eq_chars.push(name_span);
                        }

                        let mut line = String::from("");
                        for _ in 0..bands.len() {
                            line.push(EQ_POS_CHAR);                            
                        }
                        let name_span = Line::from(vec![
                            Span::raw(format!("{}", line)).blue()
                            ]);
                        eq_chars.push(name_span);

                        Paragraph::new(eq_chars)
                            .centered()
                            .block(track_eq_block)
                            .render(current_layout_info[1], buf);
                    },
                    None => {
                        Paragraph::new(Line::from(vec![
                            Span::raw("No freq. info.")
                        ]))
                            .centered()
                            .block(track_eq_block)
                            .render(current_layout_info[1], buf);
                    }
                }
        
            },

            DisplayMode::FileSelection => {
                let fs_title = Line::from(" File Selection: ").bold();

                let fs_instructions = Line::from(vec![
                    " Back ".into(),
                    "<a>".blue().bold(),
                    " Down ".into(),
                    "<s>".blue().bold(),
                    " Up ".into(),
                    "<d>".blue().bold(),
                    " Into ".into(),
                    "<f>".blue().bold(),
                    " Load ".into(),
                    "<Enter> ".blue().bold(),
                ]);

                let mut fs_lines: Vec<Line<'_>> = Vec::new();

                let selected_entry = self.file_selector.selected_entry();
                let file_entries = self.file_selector.contents();

                for n in 0..file_entries.len() {
                    let entry = file_entries.get(n).unwrap();
                    let path = entry.path();
                    let name = path.as_path().to_str().unwrap();
                    let mut span = 
                                Span::raw(format!("  {}", name));

                    if path.is_dir() {
                        span = span.add_modifier(Modifier::BOLD);
                    } 

                    if n == selected_entry {
                        span = span.blue();
                    }

                    let line = Line::from(vec![
                            span
                    ]);
                    fs_lines.push(line);
                }


                let fs_block = Block::bordered()
                    .title(fs_title.left_aligned())
                    .title_bottom(fs_instructions.centered())
                    .border_set(border::THICK);

                let scroll : isize = self.file_selector.selected_entry() as isize - 1;
                
                Paragraph::new(fs_lines)
                    .left_aligned()
                    .scroll((
                        (scroll*(scroll.is_positive() as isize)).try_into().unwrap()
                        , 0))
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
