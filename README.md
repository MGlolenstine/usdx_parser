# UltraStar Deluxe parser

## About

This is a rust parser for USDX song files.
Files are written as a plaintext files that contain data about the song and notes/lyrics.

## Usage
Direct read from a file:
```Rust
let song = Song::from_file("tests/test.txt").unwrap();
dbg!(song);
```

Read from a `&str`:
```Rust
let text = std::fs::read_to_string("tests/test.txt").unwrap();
let song = Song::from_str(&text).unwrap();
dbg!(song);
```

Parse directly from String:
```Rust
let text = r#"
#ARTIST:Three Days Grace
#TITLE:I Hate Everything About You
#MP3:i_hate_everything_about_you.ogg
#LANGUAGE:English
#BPM:100
#GAP:100
"#;
let song: Song = text.to_string().into();
dbg!(song);
```