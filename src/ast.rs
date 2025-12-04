use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Solfa {
    pub header: HashMap<String, String>,
    pub staffs: Vec<Staff>,
}

#[derive(Debug, PartialEq)]
pub enum Dynamic {
    Level { pos: u16, kind: DynamicLevel },
    Accent { pos: u16 },
    Crescendo { start: u16, end: u16 },
    Decrescendo { start: u16, end: u16 },
}

#[derive(Debug, PartialEq)]
pub enum DynamicLevel {
    FF,
    F,
    MF,
    MP,
    P,
    PP,
}

#[derive(Debug, PartialEq)]
pub enum BaseNote {
    D,
    R,
    M,
    F,
    S,
    L,
    T,
}

#[derive(Debug, PartialEq)]
pub enum NoteVariant {
    Base,
    Raised,
    Lowered,
}

#[derive(Debug, PartialEq)]
pub enum Octave {
    Base,
    Up(u8),
    Down(u8),
}

#[derive(Debug, PartialEq)]
pub struct Note {
    pub base: BaseNote,
    pub variant: NoteVariant,
    pub octave: Octave,
}

impl Note {
    pub fn with_octave_up(mut self, value: u8) -> Self {
        self.octave = Octave::Up(value);
        self
    }

    pub fn with_octave_down(mut self, value: u8) -> Self {
        self.octave = Octave::Down(value);
        self
    }
}

#[derive(Debug, PartialEq)]
pub enum BeatDivisionKind {
    Normal,
    Half,
    Quarter,
}

#[derive(Debug, PartialEq)]
pub struct BeatDivision {
    pub lhs: Box<Measure>,
    pub rhs: Box<Measure>,
    pub kind: BeatDivisionKind,
}

impl BeatDivision {
    pub fn new(kind: BeatDivisionKind, lhs: Measure, rhs: Measure) -> Self {
        BeatDivision {
            lhs: lhs.into(),
            rhs: rhs.into(),
            kind,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Measure {
    EmptyNote,
    Note(Note),
    BeatDivision(BeatDivision),
    UnderlinedMeasure(Box<Measure>),
}

#[derive(Debug, PartialEq)]
pub struct Staff {
    pub dynamics: Vec<Dynamic>,
    pub first: Vec<Measure>,
    pub second: Vec<Measure>,
    pub third: Vec<Measure>,
    pub fourth: Vec<Measure>,
    pub lyrics: Vec<LyricsTree>,
}

#[derive(Debug, PartialEq)]
pub enum LyricsChunk {
    Placeholder,
    String(String),
    Split(Box<LyricsChunk>, Box<LyricsChunk>),
    Space(Box<LyricsChunk>, Box<LyricsChunk>),
    Concat(Box<LyricsChunk>, Box<LyricsChunk>),
}

#[derive(Debug, PartialEq)]
pub enum LyricsMeasure {
    Chunk(LyricsChunk),
    Join(Box<LyricsMeasure>, Box<LyricsMeasure>),
    Concat(Box<LyricsMeasure>, Box<LyricsMeasure>),
}

#[derive(Debug, PartialEq)]
pub struct LyricsTree {
    pub prefix: String,
    pub root: LyricsMeasure,
}
