use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Solfa {
    pub header: HashMap<String, String>,
    pub staffs: Vec<Staff>,
}

#[derive(Debug, PartialEq)]
pub enum Dynamic {
    DC { pos: u16 },
    DS { pos: u16 },
    Sign { pos: u16 },
    Accent { pos: u16 },
    Crescendo { start: u16, end: u16 },
    Decrescendo { start: u16, end: u16 },
    Level { pos: u16, kind: DynamicLevel },
}

#[derive(Debug, PartialEq)]
pub enum DynamicLevel {
    FFF,
    FF,
    F,
    MF,
    MP,
    P,
    PP,
    PPP,
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
    Repeated(Box<Measure>),
    RepeatStart(Box<Measure>),
    RepeatEnd(Box<Measure>),
}

#[derive(Debug, PartialEq)]
pub struct Staff {
    pub dynamics: Vec<Dynamic>,
    pub measures: Vec<[Measure; 4]>,
    pub lyrics: Vec<IndexedLyricsSet>,
}

impl From<StaffPartial> for Staff {
    fn from(value: StaffPartial) -> Self {
        let measures = value
            .voice1
            .into_iter()
            .zip(value.voice2) // FIXME: Error handling and validation
            .zip(value.voice3)
            .zip(value.voice4)
            .map(|(((m1, m2), m3), m4)| [m1, m2, m3, m4])
            .collect();

        let lyrics = vec![
            value.lyrics1.map(|l| IndexedLyricsSet::from((0, l))),
            value.lyrics2.map(|l| IndexedLyricsSet::from((1, l))),
            value.lyrics3.map(|l| IndexedLyricsSet::from((2, l))),
            value.lyrics4.map(|l| IndexedLyricsSet::from((3, l))),
        ];

        Self {
            dynamics: value.dynamics,
            lyrics: lyrics.into_iter().flatten().collect(),
            measures,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StaffPartial {
    pub dynamics: Vec<Dynamic>,
    pub voice1: Vec<Measure>,
    pub lyrics1: Option<Vec<LyricsTree>>,
    pub voice2: Vec<Measure>,
    pub lyrics2: Option<Vec<LyricsTree>>,
    pub voice3: Vec<Measure>,
    pub lyrics3: Option<Vec<LyricsTree>>,
    pub voice4: Vec<Measure>,
    pub lyrics4: Option<Vec<LyricsTree>>,
}

#[derive(Debug, PartialEq)]
pub enum LyricsChunk {
    Placeholder,
    String(String),
    NewLineSuffixed(Box<LyricsChunk>),
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

#[derive(Debug, PartialEq)]
pub struct IndexedLyricsSet {
    pub index: u8,
    pub lyrics: Vec<LyricsTree>,
}

impl From<(u8, Vec<LyricsTree>)> for IndexedLyricsSet {
    fn from((index, lyrics): (u8, Vec<LyricsTree>)) -> Self {
        Self { index, lyrics }
    }
}
