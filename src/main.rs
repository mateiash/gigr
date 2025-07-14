// Modules
mod song;
mod player;

use crate::song::Song;
use crate::player::Player;

fn main() {
    let song : Song = Song::new("examples/music.flac");
    let song2 : Song = Song::new("examples/music2.flac");
    let player : Player = Player::new();
    player.play(&song2);
    player.play(&song);
}
