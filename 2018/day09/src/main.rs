use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    num,
};

trait CircleBuf {
    /// Rotates the buffer by `i` steps.
    /// Positive `i` means rotating the buffer clockwise.
    /// Negative `i` means rotating the buffer counter-clockwise.
    fn rotate(&mut self, i: isize);
}

impl<T> CircleBuf for VecDeque<T> {
    fn rotate(&mut self, offset: isize) {
        if offset > 0 {
            for _ in 0..offset {
                if let Some(item) = self.pop_front() {
                    self.push_back(item)
                }
            }
        } else if offset < 0 {
            let offset = offset.abs();
            for _ in 0..offset {
                if let Some(item) = self.pop_back() {
                    self.push_front(item)
                }
            }
        }
    }
}

fn parse_input(s: &str) -> Result<(u32, u32), num::ParseIntError> {
    let mut parts = s.split_whitespace();
    let players = parts.next().unwrap().parse()?;
    let hi_marble = parts.nth(5).unwrap().parse()?;
    Ok((players, hi_marble))
}

fn level1(players: u32, hi_marble: u32) -> u32 {
    let players = players as usize;
    let mut scores = vec![0; players];
    let mut circle = VecDeque::with_capacity(hi_marble as usize);
    circle.push_back(0);

    for (player, marble) in (0..players).cycle().zip(1..=hi_marble) {
        if marble % 23 == 0 {
            scores[player] += marble;
            circle.rotate(-7);
            scores[player] += circle.pop_front().unwrap();
        } else {
            circle.rotate(2);
            circle.push_front(marble);
        }
    }

    scores.into_iter().max().unwrap()
}

fn level2(players: u32, hi_marble: u32) -> u32 {
    level1(players, hi_marble * 100)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let (players, hi_marble) = parse_input(&input)?;

    let some = level1(players, hi_marble);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(players, hi_marble);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test]
    fn level1_examples() {
        assert_eq!(level1(9, 25), 32);
        assert_eq!(level1(10, 1618), 8317);
        assert_eq!(level1(13, 7999), 146373);
        assert_eq!(level1(17, 1104), 2764);
        assert_eq!(level1(21, 6111), 54718);
        assert_eq!(level1(30, 5807), 37305);
    }

    #[test]
    fn level1_regression() {
        let (players, hi_marble) = parse_input(INPUT).unwrap();
        assert_eq!(level1(players, hi_marble), 423717);
    }

    #[test]
    fn level2_regression() {
        let (players, hi_marble) = parse_input(INPUT).unwrap();
        assert_eq!(level2(players, hi_marble), 3553108197);
    }
}
