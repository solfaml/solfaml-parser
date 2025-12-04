use super::ast::*;

use winnow::{
    ascii::{alphanumeric1, digit1, multispace0, multispace1, space0},
    combinator::{alt, delimited, opt, separated, seq},
    prelude::*,
    token::{one_of, take_while},
};

pub fn solfa_parser<'s>(input: &mut &'s str) -> ModalResult<Solfa> {
    seq! {
        Solfa {
            _: multispace0,
            header: metadata_parser.map(|metadata| {
                metadata.into_iter().collect()
            }),
            _: multispace1,
            _: "---",
            _: multispace1,
            staffs: separated(1.., staff_parser, multispace1),
            _: multispace0,
        }
    }
    .parse_next(input)
}

pub fn metadata_parser<'s>(input: &mut &'s str) -> ModalResult<Vec<(String, String)>> {
    separated(
        0..,
        seq! (
            alphanumeric1.map(|s: &str|s.to_string()),
            _: space0,
            _: ":",
            _: space0,
            take_while(1.., |ch: char| ch != '\n').map(|s: &str| s.to_string()),
        ),
        "\n",
    )
    .parse_next(input)
}

pub fn staff_parser<'s>(input: &mut &'s str) -> ModalResult<Staff> {
    seq! {
        Staff {
            first: measure_parser,
            _: "\n",
            second: measure_parser,
            _: "\n",
            third: measure_parser,
            _: "\n",
            fourth: measure_parser,
            lyrics: separated(0.., seq!(_: multispace0, lyrics_tree_parser).map(|(l,)| l), multispace1)
        }
    }
    .parse_next(input)
}

pub fn base_lyrics_parser<'s>(input: &mut &'s str) -> ModalResult<LyricsChunk> {
    seq!(
        _: space0,
        take_while(1.., |ch: char| !" _<|$\n".contains(ch)),
        opt(seq!(_: "$", base_lyrics_parser)),
    )
    .map(|(lhs, rhs)| {
        let lhs = LyricsChunk::String(lhs.to_string());
        match rhs {
            Some((rhs,)) => LyricsChunk::Split(Box::new(lhs), Box::new(rhs)),
            None => lhs,
        }
    })
    .parse_next(input)
}

pub fn lyrics_chunk_parser<'s>(input: &mut &'s str) -> ModalResult<LyricsChunk> {
    seq!(
        _: space0,
        base_lyrics_parser,
        opt(seq!(one_of((' ', '_')), lyrics_chunk_parser)),
        _: space0,
    )
    .map(|(lhs, rhs)| match rhs {
        Some((sep, rhs)) => match sep {
            '_' => LyricsChunk::Concat(Box::new(lhs), rhs.into()),
            ' ' => LyricsChunk::Space(Box::new(lhs), rhs.into()),
            _ => unreachable!(),
        },
        None => lhs,
    })
    .parse_next(input)
}

pub fn lyrics_measure_parser<'s>(input: &mut &'s str) -> ModalResult<LyricsMeasure> {
    seq!(
        lyrics_chunk_parser,
        opt(seq!(alt(("|", "<|>")), lyrics_measure_parser)),
    )
    .map(|(lhs, rhs)| {
        let lhs = LyricsMeasure::Chunk(lhs);
        match rhs {
            Some((sep, rhs)) => match sep {
                "|" => LyricsMeasure::Join(Box::new(lhs), rhs.into()),
                "<|>" => LyricsMeasure::Concat(Box::new(lhs), rhs.into()),
                _ => unreachable!(),
            },
            None => lhs,
        }
    })
    .parse_next(input)
}

pub fn lyrics_tree_parser<'s>(input: &mut &'s str) -> ModalResult<LyricsTree> {
    seq! {
        LyricsTree {
            prefix: take_while(1.., |ch: char| !" |".contains(ch))
                .map(|s: &str| s.to_string()),
            root: lyrics_measure_parser,
            _: alt(("||", "|")),
        }
    }
    .parse_next(input)
}

pub fn measure_parser<'s>(input: &mut &'s str) -> ModalResult<Vec<Measure>> {
    seq!(
        _: opt("|"),
        separated(1.., normal_div_parser, "|"),
        _: alt(("||", "|")),
    )
    .map(|(m,)| m)
    .parse_next(input)
}

pub fn octave_parser<'s>(input: &mut &'s str) -> ModalResult<Octave> {
    alt((
        "'".map(|_| Octave::Up(1)),
        seq!(_: "+", digit1).map(|(d,): (&str,)| Octave::Up(d.parse().unwrap())),
        seq!(_: "-", digit1).map(|(d,): (&str,)| Octave::Down(d.parse().unwrap())),
    ))
    .parse_next(input)
}

pub fn base_note_parser<'s>(input: &mut &'s str) -> ModalResult<BaseNote> {
    one_of(('d', 'r', 'm', 'f', 's', 'l', 't'))
        .map(|note| match note {
            'd' => BaseNote::D,
            'r' => BaseNote::R,
            'm' => BaseNote::M,
            'f' => BaseNote::F,
            's' => BaseNote::S,
            'l' => BaseNote::L,
            't' => BaseNote::T,
            _ => unreachable!(),
        })
        .parse_next(input)
}

pub fn note_parser<'s>(input: &mut &'s str) -> ModalResult<Note> {
    seq! {
        Note {
            base: base_note_parser,
            variation: opt(one_of(('a', 'i'))).map(|v| match v {
                Some('a') => NoteVariant::Lowered,
                Some('i') => NoteVariant::Raised,
                _ => NoteVariant::Base,
            }),
            octave: opt(octave_parser).map(|o| o.unwrap_or(Octave::Base))
        }
    }
    .parse_next(input)
}

pub fn base_beat_parser<'s>(input: &mut &'s str) -> ModalResult<Measure> {
    seq!(
        _: space0,
        alt((
            "-".map(|_| Measure::EmptyNote),
            note_parser.map(|n| Measure::Note(n)),
            delimited("_", normal_div_parser, "_").map(|b| Measure::UnderlinedMeasure(b.into())),
        )),
        _: space0,
    )
    .map(|(b,)| b)
    .parse_next(input)
}

pub fn quarter_div_parser<'s>(input: &mut &'s str) -> ModalResult<Measure> {
    seq!(base_beat_parser, seq!(opt(","), opt(quarter_div_parser)))
        .map(|(lhs, (op, rhs))| {
            match (op, rhs) {
                (Some(_), Some(rhs)) => {
                    Measure::BeatDivision(BeatDivision::new(BeatDivisionKind::Quarter, lhs, rhs))
                }
                (Some(_), None) => {
                    match lhs {
                        Measure::Note(note) => Measure::Note(note.with_octave_down(1)),
                        _ => panic!("invalid syntax"), // FIXME: propagate error
                    }
                }
                _ => lhs,
            }
        })
        .parse_next(input)
}

pub fn half_div_parser<'s>(input: &mut &'s str) -> ModalResult<Measure> {
    seq!(quarter_div_parser, opt(seq!(_: ".", half_div_parser)))
        .map(|(lhs, rhs)| match rhs {
            Some((rhs,)) => {
                Measure::BeatDivision(BeatDivision::new(BeatDivisionKind::Half, lhs, rhs))
            }
            None => lhs,
        })
        .parse_next(input)
}

pub fn normal_div_parser<'s>(input: &mut &'s str) -> ModalResult<Measure> {
    seq!(half_div_parser, opt(seq!(_: ":", normal_div_parser)))
        .map(|(lhs, rhs)| match rhs {
            Some((rhs,)) => {
                Measure::BeatDivision(BeatDivision::new(BeatDivisionKind::Normal, lhs, rhs))
            }
            None => lhs,
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use crate::parser::solfa_parser;

    #[test]
    fn test_simple_parsing() {
        let mut source = "
title: foo
author: bar
time: 4/4
key: C
description: Hello World!

---

| d : r : m | f . s , l : t  | _d'_ ||
| d : r : m | f . s , l : t  | _d'_ ||
| d : r : m | f . s , l : t  | _d'_ ||
| d : r : m | f . s , l : t  | _d'_ ||
 
1. do re mi | fa_so la ti$e <|> do  ||
2. do re mi | fa_so la ti$e <|> do  ||
";

        let res = solfa_parser.parse(&mut source).unwrap();

        assert_eq!(&res.header["title"], "foo");
        assert_eq!(&res.header["time"], "4/4");
        assert_eq!(&res.header["description"], "Hello World!");

        let first_staff = &res.staffs[0];

        assert_eq!(first_staff.lyrics.len(), 2);
    }
}
