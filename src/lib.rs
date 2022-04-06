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
    pub genre: Option<String>,
    pub year: Option<String>,
    pub language: Option<String>,
    /// Beats per minute
    pub bpm: u32,
    /// Delay in ms before the lyrics start after song
    pub gap: u32,
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
        let notes = lines
            .filter(|a| !(a.starts_with('#') || a.starts_with('E') || a.is_empty()))
            .filter_map(|a| Note::try_from(a).ok())
            .collect::<Vec<_>>();

        let title = if let Some(a) = title {
            a
        } else {
            bail!("No title specified!");
        };

        let bpm = if let Some(a) = bpm {
            if let Ok(a) = a.parse::<u32>() {
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

        Ok(Self {
            artist,
            title,
            mp3,
            video,
            genre,
            year,
            language,
            bpm,
            gap,
            notes,
        })
    }
}

impl ToString for Song {
    fn to_string(&self) -> String {
        let mut ret = String::new();
        if let Some(artist) = self.artist.as_ref() {
            ret.push_str(&format!("#ARTIST:{}\n", artist));
        }
        ret.push_str(&format!("#TITLE:{}\n", self.title));
        if let Some(artist) = self.mp3.as_ref() {
            ret.push_str(&format!("#MP3:{}\n", artist));
        }
        if let Some(artist) = self.genre.as_ref() {
            ret.push_str(&format!("#GENRE:{}\n", artist));
        }
        if let Some(artist) = self.year.as_ref() {
            ret.push_str(&format!("#YEAR:{}\n", artist));
        }
        if let Some(artist) = self.language.as_ref() {
            ret.push_str(&format!("#LANGUAGE:{}\n", artist));
        }
        ret.push_str(&format!("#BPM:{}\n", self.bpm));
        ret.push_str(&format!("#GAP:{}\n", self.gap));
        if let Some(artist) = self.video.as_ref() {
            ret.push_str(&format!("#VIDEO:{}\n", artist));
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
    pub note_tone: Option<u32>,
    /// String content for this note
    pub lyric: Option<String>,
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
            let note_tone = splot.next().unwrap().parse::<u32>()?;
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
pub fn test_manual_serde(){
    let text = std::fs::read_to_string("tests/i_hate_everything_about_you.txt").unwrap();
    let song = Song::from_str(&text);
    assert!(song.is_ok());
    let song = song.unwrap();
    assert_eq!(text.replace("\r\n", "\n"), song.to_string());
}