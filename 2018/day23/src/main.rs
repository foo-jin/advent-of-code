use std::{
    error::Error,
    io::{self, Read, Write},
    str::FromStr,
};

macro_rules! format_err {
    ($($tt:tt)*) => { Box::<std::error::Error>::from(format!($($tt)*)) }
}

mod aoc {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let bots = parse_bots(&input)?;

    let some = level1(&bots);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&bots);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

fn parse_bots(s: &str) -> aoc::Result<Box<[NanoBot]>> {
    s.trim().lines().map(|l| l.parse()).collect()
}

fn level1(bots: &[NanoBot]) -> usize {
    let strongest = bots.iter().max_by_key(|b| b.rd).unwrap();
    bots.iter().filter(|b| strongest.contains(b.pos)).count()
}

/// The approach used here does not generalize, since it works towards a local maximum.
/// In the z3 branch a general solution can be found using (start the drums) z3. It's performance
/// however is a lot worse (takes about a full minute on my machine).
fn level2(bots: &[NanoBot]) -> u32 {
    let (mut x_min, mut x_max) = minmax(bots, 0);
    let (mut y_min, mut y_max) = minmax(bots, 1);
    let (mut z_min, mut z_max) = minmax(bots, 2);
    let dx = (x_max - x_min) as usize;

    let mut step = 1;
    while step < dx {
        step *= 2
    }

    let mut best = 0;
    let mut p = ORIGIN;
    loop {
        for x in (x_min..=x_max).step_by(step) {
            for y in (y_min..=y_max).step_by(step) {
                for z in (z_min..=z_max).step_by(step) {
                    let q = [x, y, z];
                    let k = bots.iter().filter(|b| b.contains(q)).count();
                    if k > best || (k == best && dist(q) < dist(p)) {
                        best = k;
                        p = q;
                    }
                }
            }
        }

        match step {
            1 => return dist(p) as u32,
            _ => {
                let s = step as i64;
                shift(&mut x_min, &mut x_max, p[0], s);
                shift(&mut y_min, &mut y_max, p[1], s);
                shift(&mut z_min, &mut z_max, p[2], s);
                step /= 2;
            }
        }
    }
}

fn minmax(bots: &[NanoBot], i: usize) -> (i64, i64) {
    use itertools::Itertools;
    bots.iter()
        .map(|b| b.pos[i])
        .minmax()
        .into_option()
        .unwrap()
}

fn shift(a: &mut i64, b: &mut i64, x: i64, step: i64) {
    *a = x - step;
    *b = x + step;
}

fn absdiff(a: i64, b: i64) -> u32 {
    (a - b).abs() as u32
}

fn manhattan(p: Point, q: Point) -> u32 {
    p.iter().zip(q.iter()).map(|(&a, &b)| absdiff(a, b)).sum()
}

fn dist(p: Point) -> u32 {
    manhattan(p, ORIGIN)
}

type Point = [i64; 3];
const ORIGIN: Point = [0, 0, 0];

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct NanoBot {
    pos: Point,
    rd: u32,
}

impl NanoBot {
    fn contains(&self, p: Point) -> bool {
        manhattan(self.pos, p) <= self.rd
    }
}

impl FromStr for NanoBot {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use regex::Regex;
        lazy_static::lazy_static! {
            static ref RE: Regex =
                Regex::new(
                    r"pos=<(?P<x>[-+]?[0-9]+),(?P<y>[-+]?[0-9]+),(?P<z>[-+]?[0-9]+)>,\sr=(?P<r>[0-9]+)"
                ).unwrap();
        }
        let caps = RE
            .captures(s)
            .ok_or_else(|| format_err!("invalid input format: {}", s))?;
        let pos = [caps["x"].parse()?, caps["y"].parse()?, caps["z"].parse()?];
        let rd = caps["r"].parse()?;
        Ok(NanoBot { pos, rd })
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
    const EX1: &str = "
pos=<0,0,0>, r=4
pos=<1,0,0>, r=1
pos=<4,0,0>, r=3
pos=<0,2,0>, r=1
pos=<0,5,0>, r=3
pos=<0,0,3>, r=1
pos=<1,1,1>, r=1
pos=<1,1,2>, r=1
pos=<1,3,1>, r=1";

    #[test_log::new]
    fn regression() -> aoc::Result<()> {
        let bots = parse_bots(INPUT)?;
        assert_eq!(level1(&bots), 232, "level 1 regressed");
        assert_eq!(level2(&bots), 82010396, "level 2 regressed");
        Ok(())
    }

    #[test_log::new]
    fn level1_examples() -> aoc::Result<()> {
        let bots = parse_bots(EX1)?;
        assert_eq!(level1(&bots), 7);
        Ok(())
    }

    const EX2: &str = "
pos=<10,12,12>, r=2
pos=<12,14,12>, r=2
pos=<16,12,12>, r=4
pos=<14,14,14>, r=6
pos=<50,50,50>, r=200
pos=<10,10,10>, r=5";

    #[test_log::new]
    fn level2_examples() -> aoc::Result<()> {
        let bots = parse_bots(EX2)?;
        assert_eq!(level2(&bots), 36);
        Ok(())
    }
}
