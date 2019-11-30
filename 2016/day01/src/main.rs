use std::{
    collections::HashSet,
    io::{self, Read, Write},
};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn std::error::Error>::from(format!($($tt)*))) }
}

mod aoc {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}

type Pos = (i32, i32);

#[derive(Copy, Clone)]
enum Direction {
    North,
    South,
    East,
    West,
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let some = level1(&input)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&input)?;
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", some)?;
    Ok(())
}

fn level1(s: &str) -> aoc::Result<u32> {
    let mut pos = (0, 0);
    let mut dir = Direction::North;
    for cmd in s.split(", ") {
        let (d, p) = step(pos, dir, cmd)?;
        pos = p;
        dir = d;
    }
    let (x, y) = pos;
    Ok((x.abs() + y.abs()) as u32)
}

fn level2(s: &str) -> aoc::Result<u32> {
    let mut seen = HashSet::new();
    let mut pos = (0i32, 0i32);
    let mut dir = Direction::North;
    seen.insert(pos);

    loop {
        for cmd in s.split(", ") {
            let (d, p) = step(pos, dir, cmd)?;
            dir = d;
            while pos != p {
                pos = mv(pos, dir, 1);
                if !seen.insert(pos) {
                    let (x, y) = pos;
                    return Ok((x.abs() + y.abs()) as u32);
                }
            }
        }
    }
}

fn step(mut pos: Pos, mut dir: Direction, cmd: &str) -> aoc::Result<(Direction, Pos)> {
    use Direction::*;

    let (turn, distance) = cmd.trim().split_at(1);
    dir = match (dir, turn) {
        (North, "L") => West,
        (North, "R") => East,
        (East, "L") => North,
        (East, "R") => South,
        (South, "L") => East,
        (South, "R") => West,
        (West, "R") => North,
        (West, "L") => South,
        _ => err!("Unknown turn: {}", turn)?,
    };

    let distance = distance.parse::<i32>()?;
    pos = mv(pos, dir, distance);

    Ok((dir, pos))
}

fn mv(pos: Pos, dir: Direction, distance: i32) -> Pos {
    use Direction::*;

    let (x, y) = pos;
    match dir {
        North => (x, y + distance),
        South => (x, y - distance),
        East => (x + distance, y),
        West => (x - distance, y),
    }
}

fn main() -> aoc::Result<()> {
    env_logger::init();
    if let Err(e) = solve() {
        let stderr = io::stderr();
        let mut w = stderr.lock();
        writeln!(w, "Error: {}", e)?;
        while let Some(e) = e.source() {
            writeln!(w, "\t{}", e)?;
        }

        std::process::exit(-1)
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test_log::new]
    fn level1_examples() -> aoc::Result<()> {
        assert_eq!(level1("R2, L3")?, 5);
        assert_eq!(level1("R2, R2, R2")?, 2);
        assert_eq!(level1("R2, L3")?, 5);
        assert_eq!(level1("R5, L5, R5, R3")?, 12);
        Ok(())
    }

    #[test]
    fn level1_sanity() -> aoc::Result<()> {
        assert_eq!(level1(INPUT)?, 278);
        Ok(())
    }

    #[test]
    fn level2_examples() -> aoc::Result<()> {
        assert_eq!(level2("R8, R4, R4, R8")?, 4);
        Ok(())
    }

    #[test]
    fn level2_sanity() -> aoc::Result<()> {
        assert_eq!(level2(INPUT)?, 161);
        Ok(())
    }
}
