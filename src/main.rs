// Modules
mod song;
mod player;

use crate::song::Song;
use crate::player::Player;

fn main() {
    let song : Song = Song::new("examples/music.flac");
    let song2 : Song = Song::new("examples/music2.flac");
    let player : Player = Player::new();
    player.add_to_queue(&song);
    player.add_to_queue(&song2);
    player.sleep_until_end();
}
