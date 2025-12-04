use std::collections::HashMap;

#[derive(Debug)]
pub struct Solfa {
    pub header: HashMap<String, String>,
    pub staffs: Vec<Staff>,
}

#[derive(Debug)]
pub enum BaseNote {
    D,
    R,
    M,
    F,
    S,
    L,
    T,
}

#[derive(Debug)]
pub enum NoteVariant {
    Base,
    Raised,
    Lowered,
}

#[derive(Debug)]
pub enum Octave {
    Base,
    Up(u8),
    Down(u8),
}

#[derive(Debug)]
pub struct Note {
    pub base: BaseNote,
    pub variation: NoteVariant,
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

#[derive(Debug)]
pub enum BeatDivisionKind {
    Normal,
    Half,
    Quarter,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Measure {
    EmptyNote,
    Note(Note),
    EmphasedNote(Note),
    BeatDivision(BeatDivision),
    UnderlinedMeasure(Box<Measure>),
}

#[derive(Debug)]
pub struct Staff {
    pub first: Vec<Measure>,
    pub second: Vec<Measure>,
    pub third: Vec<Measure>,
    pub fourth: Vec<Measure>,
    pub lyrics: Vec<LyricsTree>,
}

#[derive(Debug)]
pub enum LyricsChunk {
    String(String),
    Split(Box<LyricsChunk>, Box<LyricsChunk>),
    Space(Box<LyricsChunk>, Box<LyricsChunk>),
    Concat(Box<LyricsChunk>, Box<LyricsChunk>),
}

#[derive(Debug)]
pub enum LyricsMeasure {
    Chunk(LyricsChunk),
    Join(Box<LyricsMeasure>, Box<LyricsMeasure>),
    Concat(Box<LyricsMeasure>, Box<LyricsMeasure>),
}

#[derive(Debug)]
pub struct LyricsTree {
    pub prefix: String,
    pub root: LyricsMeasure,
}
