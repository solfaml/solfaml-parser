use super::ast::*;

use winnow::{
    ascii::{alphanumeric1, digit1, multispace0, multispace1, space0, space1},
    combinator::{alt, delimited, not, opt, repeat, separated, seq},
    prelude::*,
    token::{one_of, take_while},
};

pub fn solfa_parser(input: &mut &str) -> ModalResult<Solfa> {
    seq! {
        Solfa {
            _: multispace0,
            header: metadata_parser.map(|metadata| {
                metadata.into_iter().collect()
            }),
            _: multispace0,
            _: "---",
            _: multispace1,
            staffs: separated(1.., staff_parser, multispace1),
            _: multispace0,
        }
    }
    .parse_next(input)
}

pub fn metadata_parser(input: &mut &str) -> ModalResult<Vec<(String, String)>> {
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

pub fn dynamics_parser(input: &mut &str) -> ModalResult<Vec<Dynamic>> {
    seq!(
        opt("|:"),
        _: space0,
        separated(0.., dynamic_base_parser, space1),
        _: space0,
        _: opt(alt(("||", "|"))),
        _: opt("\n"),
    )
    .map(|(prefix, e)| prefix.map(|_| e).unwrap_or_default())
    .parse_next(input)
}

pub fn staff_parser(input: &mut &str) -> ModalResult<Staff> {
    seq! {
        StaffPartial {
            dynamics: dynamics_parser,
            _: opt(staff_bar_parser),
            voice1: measure_parser,
            lyrics1: opt(lyrics_parser),
            voice2: measure_parser,
            lyrics2: opt(lyrics_parser),
            voice3: measure_parser,
            lyrics3: opt(lyrics_parser),
            voice4: measure_parser,
            lyrics4: opt(lyrics_parser),
        }
    }
    .map(Staff::from)
    .parse_next(input)
}

pub fn staff_bar_parser(input: &mut &str) -> ModalResult<()> {
    seq!(
        _: "|",
        _: take_while(1.., |ch: char| ch == '-'),
        _: alt(("||", "|")),
        _: "\n"
    )
    .parse_next(input)
}

pub fn lyrics_parser(input: &mut &str) -> ModalResult<Vec<LyricsTree>> {
    separated(
        1..,
        seq!(
            _: multispace0,
            lyrics_tree_parser,
        )
        .map(|(l,)| l),
        multispace1,
    )
    .parse_next(input)
}

pub fn pos_parser(input: &mut &str) -> ModalResult<u16> {
    seq!(_: "{", _: space0, digit1, _: space0, _: "}")
        .try_map(|(pos,): (&str,)| pos.parse())
        .parse_next(input)
}

pub fn range_parser(input: &mut &str) -> ModalResult<(u16, u16)> {
    seq!(_: "{", _: space0, digit1,_: ",", digit1, _: space0, _: "}")
        .try_map(|(start, end): (&str, &str)| {
            start.parse().and_then(|s| end.parse().map(|e| (s, e)))
        })
        .parse_next(input)
}

pub fn dynamic_level_parser(input: &mut &str) -> ModalResult<Dynamic> {
    seq!(
        alt((
            "fff".map(|_| DynamicLevel::FFF),
            "ff".map(|_| DynamicLevel::FF),
            "f" .map(|_| DynamicLevel::F),
            "mf".map(|_| DynamicLevel::MF),
            "mp".map(|_| DynamicLevel::MP),
            "p" .map(|_| DynamicLevel::P),
            "pp".map(|_| DynamicLevel::PP),
            "ppp".map(|_| DynamicLevel::PPP),
        )),
        _: space0,
        pos_parser
    )
    .map(|(kind, pos)| Dynamic::Level { kind, pos })
    .parse_next(input)
}

pub fn dynamic_base_parser(input: &mut &str) -> ModalResult<Dynamic> {
    alt((
        dynamic_level_parser,
        seq!(_: "DC", _: space0, pos_parser).map(|(pos,)| Dynamic::DC { pos }),
        seq!(_: "^", _: space0, pos_parser).map(|(pos,)| Dynamic::Accent { pos }),
        seq!(_: "<", _: space0, range_parser)
            .map(|((start, end),)| Dynamic::Crescendo { start, end }),
        seq!(_: ">", _: space0, range_parser)
            .map(|((start, end),)| Dynamic::Decrescendo { start, end }),
    ))
    .parse_next(input)
}

pub fn base_lyrics_parser(input: &mut &str) -> ModalResult<LyricsChunk> {
    seq!(
        alt((
            "%".map(|_| LyricsChunk::Placeholder),
            seq!(
                take_while(1.., |ch: char| !" _<|$\n%\\".contains(ch)),
                opt(seq!(_: "$", base_lyrics_parser)),
            )
            .map(|(lhs, rhs)| {
                let lhs = LyricsChunk::String(lhs.to_string());
                match rhs {
                    Some((rhs,)) => LyricsChunk::Split(Box::new(lhs), Box::new(rhs)),
                    None => lhs,
                }
            }),
        )),
        opt("\\")
    )
    .map(|(lyrics, newline)| match newline.is_some() {
        false => lyrics,
        true => LyricsChunk::NewLineSuffixed(Box::new(lyrics)),
    })
    .parse_next(input)
}

pub fn lyrics_chunk_parser(input: &mut &str) -> ModalResult<LyricsChunk> {
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

pub fn lyrics_measure_parser(input: &mut &str) -> ModalResult<LyricsMeasure> {
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

pub fn lyrics_tree_parser(input: &mut &str) -> ModalResult<LyricsTree> {
    seq! {
        LyricsTree {
            prefix: take_while(1.., |ch: char| !" |".contains(ch))
                .map(|s: &str| s.to_string()),
            root: lyrics_measure_parser,
            _: opt(alt(("||", "|"))),
        }
    }
    .parse_next(input)
}

pub fn measure_parser(input: &mut &str) -> ModalResult<Vec<Measure>> {
    seq!(
        _: multispace0,
        _: opt("|"),
        separated(1.., normal_div_parser, "|"),
        _: alt(("||", "|")),
    )
    .map(|(m,)| m)
    .parse_next(input)
}

pub fn octave_parser(input: &mut &str) -> ModalResult<Octave> {
    alt((
        seq!(_: "+", digit1)
            .try_map(|(d,): (&str,)| d.parse())
            .map(Octave::Up),
        seq!(_: "-", digit1)
            .try_map(|(d,): (&str,)| d.parse())
            .map(Octave::Down),
        seq!(repeat(1.., ','), _: not(normal_div_parser))
            .try_map(|(s,): (Vec<char>,)| s.len().try_into())
            .map(Octave::Down),
        repeat(1.., '\'')
            .try_map(|s: Vec<char>| s.len().try_into())
            .map(Octave::Up),
    ))
    .parse_next(input)
}

pub fn base_note_parser(input: &mut &str) -> ModalResult<BaseNote> {
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

pub fn note_parser(input: &mut &str) -> ModalResult<Note> {
    seq! {
        Note {
            base: base_note_parser,
            variant: opt(one_of(('a', 'i'))).map(|v| match v {
                Some('a') => NoteVariant::Lowered,
                Some('i') => NoteVariant::Raised,
                _ => NoteVariant::Base,
            }),
            octave: opt(octave_parser).map(|o| o.unwrap_or(Octave::Base))
        }
    }
    .parse_next(input)
}

pub fn base_beat_parser(input: &mut &str) -> ModalResult<Measure> {
    seq!(
        _: space0,
        alt((
            "-".map(|_| Measure::EmptyNote),
            note_parser.map(Measure::Note),
            delimited("_", normal_div_parser, "_").map(|b| Measure::UnderlinedMeasure(b.into())),
        )),
        _: space0,
    )
    .map(|(b,)| b)
    .parse_next(input)
}

pub fn quarter_div_parser(input: &mut &str) -> ModalResult<Measure> {
    seq!(base_beat_parser, opt(seq!(_: ",", quarter_div_parser)))
        .map(|(lhs, rhs)| match rhs {
            Some((rhs,)) => {
                Measure::BeatDivision(BeatDivision::new(BeatDivisionKind::Quarter, lhs, rhs))
            }
            _ => lhs,
        })
        .parse_next(input)
}

pub fn half_div_parser(input: &mut &str) -> ModalResult<Measure> {
    seq!(quarter_div_parser, opt(seq!(_: ".", half_div_parser)))
        .map(|(lhs, rhs)| match rhs {
            Some((rhs,)) => {
                Measure::BeatDivision(BeatDivision::new(BeatDivisionKind::Half, lhs, rhs))
            }
            None => lhs,
        })
        .parse_next(input)
}

pub fn normal_div_parser(input: &mut &str) -> ModalResult<Measure> {
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

    use crate::parser::{
        dynamics_parser, lyrics_tree_parser, measure_parser, metadata_parser, note_parser,
        solfa_parser,
    };

    #[test]
    fn test_metadata_parser() {
        let source = "title: foo
author: bar
time: 4/4
key: C
description: Hello World!";

        let metadata = metadata_parser.parse(source);

        insta::assert_debug_snapshot!(metadata);
    }

    #[test]
    fn test_dynamics_parsing() {
        let source = "|: f{1} <{3,7} ^{8} mp{10} ||";
        let dynamics = dynamics_parser.parse(source).unwrap();

        insta::assert_debug_snapshot!(dynamics);
    }

    #[test]
    fn test_note_parsing() {
        let source = [
            "d", "r", "m", "f", "s", "l", "t", "d'", "r,", "m+2", "f-2", "ti", "da", "ri'", "ma,",
            "si+1", "ra-3", "d,,", "r''",
        ];

        let notes = source
            .into_iter()
            .map(|s| note_parser.parse(s))
            .collect::<Vec<_>>();

        insta::assert_debug_snapshot!(notes);
    }

    #[test]
    fn test_measure_parsing() {
        let source = "| d : r .  m , f  | s : _l . t_ , - ||";
        let measure = measure_parser.parse(source);

        insta::assert_debug_snapshot!(measure);
    }

    #[test]
    fn test_lyrics_parsing() {
        let source = "1. do re_mi\\ | fasola ti$e <|> do % ||";
        let lyrics = lyrics_tree_parser.parse(source);

        insta::assert_debug_snapshot!(lyrics);
    }

    #[test]
    fn test_simple_solfa_parsing() {
        let source = "
---
| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||
";

        let result = solfa_parser.parse(source);

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_per_voice_lyrics_parsing() {
        let source = "
---
| d : r ||
| d : r ||
> do re
| d : r ||
| d : r ||
> doo ree
";

        let result = solfa_parser.parse(source);

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_multi_staff_parsing() {
        let source = "
---
| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||

> do re | mi fa

| s : l | t : d' ||
| s : l | t : d' ||

> so la | ti do

| s : l | t : - ||
| s : l | t : - ||

> so la | ti
";

        let result = solfa_parser.parse(source);

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_full_parsing() {
        let source = "
title: foo
author: bar
time: 4/4
key: C
description: Hello World!

---

|: p{1}       <{4,7}             ^{8}   DC{9} ||
|---------------------------------------------||
| d : r : m | f . s , l :  t   | _d'_ : ri+2  ||
| d : r : m | f . s , l :  t   | _d'_ : ri+2  ||
| d : r : m | f . s , l :  t   | _d'_ : ra-1  ||
| d : r : m | f . s , l :  t   |  d,  : ra-1  ||

1. do re_mi |   fasola   ti$e <|> do     re   ||
2. do re_mi |   fasola   ti$e <|> do     %    ||
";

        let solfa = solfa_parser.parse(source).unwrap();

        insta::assert_debug_snapshot!(solfa.staffs);
    }
}
