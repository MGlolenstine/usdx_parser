use anyhow::{bail, Result};
use std::str::FromStr;

/// Song information
#[derive(Debug, Clone)]
pub struct Song {
    pub artist: Option<String>,
    pub title: String,
    /// Path to the audio file
    pub mp3: Option<String>,
    pub video: Option<String>,
    pub edition: Option<String>,
    pub genre: Option<String>,
    pub year: Option<String>,
    pub language: Option<String>,
    /// Beats per minute
    pub bpm: f32,
    /// Delay in ms before the lyrics start after song
    pub gap: u32,
    pub video_gap: Option<u32>,
    /// All notes with lyrics
    pub notes: Vec<Note>,
}

impl TryFrom<String> for Song {
    type Error = anyhow::Error;

    /// ```
    /// use usdx_parser::Song;
    /// use anyhow::Result;
    ///
    /// let text = r#"
    /// #ARTIST:Three Days Grace
    /// #TITLE:I Hate Everything About You
    /// #MP3:i_hate_everything_about_you.ogg
    /// #LANGUAGE:English
    /// #BPM:100
    /// #GAP:100
    /// "#;
    /// let song: Result<Song> = text.to_string().try_into();
    /// assert!(song.is_ok());
    /// ```
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let lines = value.lines().map(|a| a.trim_start());
        let artist = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#ARTIST:"))
            .map(|a| a.to_string())
            .next();
        let title = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#TITLE:"))
            .map(|a| a.to_string())
            .next();
        let mp3 = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#MP3:"))
            .map(|a| a.to_string())
            .next();
        let video = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#VIDEO:"))
            .map(|a| a.to_string())
            .next();
        let edition = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#EDITION:"))
            .map(|a| a.to_string())
            .next();
        let genre = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#GENRE:"))
            .map(|a| a.to_string())
            .next();
        let year = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#YEAR:"))
            .map(|a| a.to_string())
            .next();
        let language = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#LANGUAGE:"))
            .map(|a| a.to_string())
            .next();
        let bpm = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#BPM:"))
            .map(|a| a.to_string())
            .next();
        let gap = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#GAP:"))
            .map(|a| a.to_string())
            .next();
        let video_gap = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#VIDEOGAP:"))
            .map(|a| a.to_string())
            .next();
        let relative = lines
            .clone()
            .filter_map(|l| l.strip_prefix("#RELATIVE:"))
            .map(|a| a.to_string())
            .next()
            .unwrap_or("no".to_string());
        let relative = parse_yes_no(&relative);
        let mut counter = 0;
        let notes = lines
            .filter(|a| !(a.starts_with('#') || a.starts_with('E') || a.is_empty()))
            .filter_map(|a| Note::try_from(a).ok())
            .map(|mut note| {
                if relative {
                    if let Some(offset) = note.update_offset() {
                        note.offset(counter);
                        counter += offset;
                    } else {
                        note.offset(counter);
                    }
                }
                note
            })
            .collect::<Vec<_>>();

        let title = if let Some(a) = title {
            a
        } else {
            bail!("No title specified!");
        };

        let bpm = if let Some(a) = bpm {
            let a = a.replace(',', ".");
            if let Ok(a) = a.parse::<f32>() {
                a
            } else {
                bail!("BPM specified failed to be parsed!");
            }
        } else {
            bail!("No bpm specified!");
        };

        let gap = if let Some(a) = gap {
            a.parse::<u32>()?
        } else {
            bail!("No gap specified!");
        };

        let video_gap = if let Some(a) = video_gap {
            Some(a.parse::<u32>()?)
        } else {
            None
        };

        Ok(Self {
            artist,
            title,
            mp3,
            video,
            edition,
            genre,
            year,
            language,
            bpm,
            gap,
            video_gap,
            notes,
        })
    }
}

fn parse_yes_no(input: &str) -> bool {
    match input {
        "yes" | "true" => true,
        "no" | "false" => false,
        _ => unimplemented!(),
    }
}

impl ToString for Song {
    fn to_string(&self) -> String {
        let mut ret = String::new();
        if let Some(artist) = self.artist.as_ref() {
            ret.push_str(&format!("#ARTIST:{}\n", artist));
        }
        ret.push_str(&format!("#TITLE:{}\n", self.title));
        if let Some(mp3) = self.mp3.as_ref() {
            ret.push_str(&format!("#MP3:{}\n", mp3));
        }
        if let Some(edition) = self.edition.as_ref() {
            ret.push_str(&format!("#EDITION:{}\n", edition));
        }
        if let Some(genre) = self.genre.as_ref() {
            ret.push_str(&format!("#GENRE:{}\n", genre));
        }
        if let Some(year) = self.year.as_ref() {
            ret.push_str(&format!("#YEAR:{}\n", year));
        }
        if let Some(language) = self.language.as_ref() {
            ret.push_str(&format!("#LANGUAGE:{}\n", language));
        }
        ret.push_str(&format!(
            "#BPM:{}\n",
            self.bpm.to_string().replace('.', ",")
        ));
        ret.push_str(&format!("#GAP:{}\n", self.gap));
        if let Some(video) = self.video.as_ref() {
            ret.push_str(&format!("#VIDEO:{}\n", video));
        }
        if let Some(video_gap) = self.video_gap.as_ref() {
            ret.push_str(&format!("#VIDEOGAP:{}\n", video_gap));
        }
        for n in self.notes.iter() {
            ret.push_str(&n.to_string());
            ret.push('\n');
        }
        ret.push_str("E\n");
        ret
    }
}

impl Song {
    /// Parse song from file
    /// ```rust
    /// use usdx_parser::Song;
    ///
    /// let song = Song::from_file("tests/i_hate_everything_about_you.txt");
    /// assert!(song.is_ok());
    /// ```
    pub fn from_file(path: &str) -> Result<Song> {
        let string = std::fs::read_to_string(path)?;
        Song::try_from(string)
    }
}

impl FromStr for Song {
    type Err = anyhow::Error;

    /// ```rust
    /// use usdx_parser::Song;
    /// use std::str::FromStr;
    ///
    /// let text = std::fs::read_to_string("tests/i_hate_everything_about_you.txt").unwrap();
    /// let song = Song::from_str(&text);
    /// assert!(song.is_ok());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_string().try_into()
    }
}

/// Note information
#[derive(Debug, Clone)]
pub struct Note {
    pub note_type: NoteType,
    /// Number of beats after start of the song when this note happens
    pub beat_number: u32,
    /// Number of beats this note lasts
    pub note_length: Option<u32>,
    pub note_tone: Option<i32>,
    /// String content for this note
    pub lyric: Option<String>,
}

impl Note {
    // Updates the offset if the note is LineBreak
    pub fn update_offset(&self) -> Option<u32> {
        if self.note_type == NoteType::LineBreak {
            Some(self.beat_number)
        } else {
            None
        }
    }

    // Offsets the note by `n` beats.
    // Used for relative lyrics
    pub fn offset(&mut self, n: u32) {
        self.beat_number += n;
    }
}

impl TryFrom<&str> for Note {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut splot = value.split(' ');
        let note_type = splot.next().unwrap().try_into()?;
        let beat_number = splot.next().unwrap().parse::<u32>()?;
        let (note_length, note_tone, lyric) = if note_type == NoteType::LineBreak {
            (None, None, None)
        } else {
            let note_length = splot.next().unwrap().parse::<u32>()?;
            let note_tone = splot.next().unwrap().parse::<i32>()?;
            let lyric = splot.collect::<Vec<_>>().join(" ");
            (Some(note_length), Some(note_tone), Some(lyric))
        };
        Ok(Self {
            note_type,
            beat_number,
            note_length,
            note_tone,
            lyric,
        })
    }
}

impl ToString for Note {
    fn to_string(&self) -> String {
        match self.note_type {
            NoteType::LineBreak => format!("{} {}", self.note_type.to_string(), self.beat_number),
            _ => format!(
                "{} {} {} {} {}",
                self.note_type.to_string(),
                self.beat_number,
                self.note_length.unwrap(),
                self.note_tone.unwrap(),
                self.lyric.as_ref().unwrap()
            ),
        }
    }
}

/// Type of the note present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteType {
    Normal,
    Golden,
    Freestyle,
    LineBreak,
}

impl TryFrom<&str> for NoteType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            ":" => Self::Normal,
            "*" => Self::Golden,
            "F" => Self::Freestyle,
            "-" => Self::LineBreak,
            _ => bail!("Unknown note type: {}", value),
        })
    }
}

impl ToString for NoteType {
    fn to_string(&self) -> String {
        match self {
            Self::Normal => ":",
            Self::Golden => "*",
            Self::Freestyle => "F",
            Self::LineBreak => "-",
        }
        .to_string()
    }
}

#[test]
pub fn test_manual_serde() {
    let text = std::fs::read_to_string("tests/queen_bohemian_rhapsody.txt").unwrap();
    let song = Song::from_str(&text);
    assert!(song.is_ok());
    let song = song.unwrap();
    assert_eq!(text.replace("\r\n", "\n"), song.to_string());
}

#[test]
pub fn test_manual_serde_relative() {
    let text = std::fs::read_to_string("tests/please_tell_rosie.txt").unwrap();
    let song = Song::from_str(&text);
    assert!(song.is_ok());
    let song = song.unwrap();
    // dbg!(song);
    println!("{}", song.to_string());
}
