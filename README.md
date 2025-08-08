# gigr

```
         oo                   
                              
.d8888b. dP .d8888b. 88d888b. 
88'  `88 88 88'  `88 88'  `88 
88.  .88 88 88.  .88 88       
`8888P88 dP `8888P88 dP       
     .88         .88          
 d8888P      d8888P by mateiash
```

`gigr` is a command-line interface tool for playing music, written in Rust. It's simple, but development has not yet been completed :D! There are more features planned.

The program allows you to select directories from which to play supported audio formats (`flac`, `wav` and `mp3`), adding them in a queue. If the directory of the currently playing song contains a `Cover.jpg` file, album artwork will be displayed.

## Features:
* Queue-based playback
* Album art display
* Multiple navigation modes
* Simple keyboard controls
* GNU/Linux support

## Commands:
### Basic playback:
* `h` - jump to previous track
* `j` - volume down
* `k` - volume up
* `l` - skip track
* `space` - play/pause
### Mode switching:
* `i` - File Selector Mode
* `o` - Queue View Mode
* `p` - Track Info Mode
* `q` - quit
### Navigation inside the File Selector Mode:
* `a` - move to the parent directory
* `s` - move down
* `d` - move up
* `f` - move inside selected directory
* `Enter` - add files inside selected directory to the queue

(Support for adding individual files to the queue is coming soon!)

### Building

This project is intended for use on GNU/Linux systems and can be built with Cargo.
To build, ensure you have Rust installed, then run:

```
cargo build --release
```

Thanks!