#[macro_use]
extern crate nom;
use failure::format_err;
use std::{
    cmp,
    collections::HashSet,
    io::{self, Read, Write},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Point {
    x: u32,
    y: u32,
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<cmp::Ordering> {
        let x = self.x.cmp(&other.x);
        let y = self.y.cmp(&other.y);

        if x == y {
            Some(x)
        } else {
            None
        }
    }
}

impl Point {
    fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Claim {
    id: u32,
    rect: Rectangle,
}

impl FromStr for Claim {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use nom::{digit, types::CompleteStr as NomInput};

        named!(parse_id(NomInput) -> u32,
            map_res!(preceded!(char!('#'), digit), |d: NomInput| d.parse())
        );

        named!(corner(NomInput) -> Point,
            map!(
                separated_pair!(
                    map_res!(digit, |d: NomInput| d.parse()),
                    char!(','),
                    map_res!(digit, |d: NomInput| d.parse())
                ),
                |(x, y)| Point::new(x, y)
            )
        );

        named!(claim(NomInput) -> Claim,
            do_parse!(
                id: parse_id >>
                ws!(char!('@')) >>
                top_left: corner >>
                ws!(char!(':')) >>
                width: map_res!(terminated!(digit, tag!("x")), |d: NomInput| d.parse()) >>
                height: map_res!(digit, |d: NomInput| d.parse()) >>
                (Claim { id, rect: Rectangle::new(top_left, width, height)})
            )
        );

        claim(NomInput(s))
            .map(|(_rest, result)| result)
            .map_err(|e| format_err!("Failed to parse square: {}", e))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
    width: u32,
    height: u32,
}

impl Rectangle {
    fn new(top_left: Point, width: u32, height: u32) -> Self {
        let Point { x, y } = top_left;
        let bottom_right = Point::new(x + width, y + height);
        Rectangle {
            top_left,
            bottom_right,
            width,
            height,
        }
    }

    fn iter(self) -> impl Iterator<Item = Point> {
        use std::iter;

        let Point { x: left, y: top } = self.top_left;
        let Point {
            x: right,
            y: bottom,
        } = self.bottom_right;
        (top..bottom)
            .flat_map(move |y| iter::repeat(y).zip(left..right))
            .map(|(y, x)| Point::new(x, y))
    }

    fn overlaps(&self, other: &Rectangle) -> Option<Rectangle> {
        if self.top_left <= other.bottom_right && other.top_left <= self.bottom_right {
            let left = self.top_left.x.max(other.top_left.x);
            let right = self.bottom_right.x.min(other.bottom_right.x);
            let top = self.top_left.y.max(other.top_left.y);
            let bottom = self.bottom_right.y.min(other.bottom_right.y);

            let top_left = Point::new(left, top);
            let width = right - left;
            let height = bottom - top;

            let overlap = Rectangle::new(top_left, width, height);
            Some(overlap)
        } else {
            None
        }
    }
}

fn parse_claims(s: &str) -> Result<Vec<Claim>, failure::Error> {
    s.lines().map(str::parse::<Claim>).collect()
}

fn level1(claims: &[Claim]) -> u32 {
    let mut overlapped = HashSet::new();
    for (i, a) in claims.iter().enumerate() {
        for b in claims.iter().skip(i + 1) {
            if let Some(overlap) = a.rect.overlaps(&b.rect) {
                overlapped.extend(overlap.iter());
            }
        }
    }
    overlapped.len() as u32
}

fn level2(claims: &[Claim]) -> Option<u32> {
    let mut intact = claims.iter().map(|c| c.id).collect::<HashSet<u32>>();
    for (i, a) in claims.iter().enumerate() {
        for b in claims.iter().skip(i + 1) {
            if a.rect.overlaps(&b.rect).is_some() {
                intact.remove(&a.id);
                intact.remove(&b.id);
            }
        }
    }
    intact.into_iter().next()
}

fn main() -> Result<(), failure::Error> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let claims = parse_claims(&input)?;

    let overlap = level1(&claims);
    writeln!(io::stderr(), "level 1: {}", overlap)?;

    let intact = level2(&claims).unwrap();
    writeln!(io::stderr(), "level 2: {}", intact)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", intact)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test]
    fn claim_parse() {
        let input = "#123 @ 3,2: 5x4";
        let expected = Claim {
            id: 123,
            rect: Rectangle::new(Point { x: 3, y: 2 }, 5, 4),
        };
        let result = input.parse::<Claim>().unwrap();
        assert_eq!(result, expected)
    }

    #[test]
    fn level1_examples() {
        let input = "#1 @ 1,3: 4x4\n#2 @ 3,1: 4x4\n#3 @ 5,5: 2x2";
        let squares = parse_claims(input).unwrap();
        assert_eq!(level1(&squares), 4)
    }

    #[test]
    fn level2_examples() {
        let input = "#1 @ 1,3: 4x4\n#2 @ 3,1: 4x4\n#3 @ 5,5: 2x2";
        let squares = parse_claims(input).unwrap();
        assert_eq!(level2(&squares), Some(3))
    }

    #[test]
    fn level1_regression() {
        let claims = parse_claims(INPUT).unwrap();
        assert_eq!(level1(&claims), 113576);
    }

    #[test]
    fn level2_regression() {
        let claims = parse_claims(INPUT).unwrap();
        assert_eq!(level2(&claims), Some(825));
    }
}
