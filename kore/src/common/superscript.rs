const MARK: char = '^';
const ENDS: [char; 2] = [' ', '~'];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Segment<'a> {
    Plain(&'a str),
    Raised(&'a str),
}

pub fn segments(line: &str) -> Vec<Segment<'_>> {
    let mut parts = line.split(MARK);
    let mut out = Vec::new();

    if let Some(first) = parts.next()
        && !first.is_empty()
    {
        out.push(Segment::Plain(first));
    }

    for part in parts {
        let (raised, plain) = part.find(ENDS).map_or((part, ""), |end| part.split_at(end));

        if !raised.is_empty() {
            out.push(Segment::Raised(raised));
        }
        if !plain.is_empty() {
            out.push(Segment::Plain(plain));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::{segments, Segment};

    #[test]
    fn a_tilde_ends_the_superscript_like_a_space_does() {
        assert_eq!(
            segments("1.00s^30f~2.00s^60f"),
            [Segment::Plain("1.00s"), Segment::Raised("30f"), Segment::Plain("~2.00s"), Segment::Raised("60f")]
        );
        assert_eq!(segments("5.00s^150f for"), [Segment::Plain("5.00s"), Segment::Raised("150f"), Segment::Plain(" for")]);
        assert_eq!(segments("no marks"), [Segment::Plain("no marks")]);
    }
}
