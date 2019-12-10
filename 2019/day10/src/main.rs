use std::{
    collections::HashSet,
    io::{self, Read, Write},
};

type Point = (usize, usize);

fn parse(s: &str) -> aoc::Result<Vec<Point>> {
    let meteors = s
        .trim()
        .lines()
        .enumerate()
        .map(|(y, l)| {
            l.chars().enumerate().filter_map(move |(x, c)| match c {
                '#' => Some((x, y)),
                _ => None,
            })
        })
        .flatten()
        .collect();
    Ok(meteors)
}

fn level1(meteors: &[Point]) -> aoc::Result<u32> {
    for &(x1, y1) in meteors {
        let mut angles = HashSet::new();
        for &(x2, y2) in meteors {
            if (x2, y2) == (x1, y1) {
                continue;
            }
            let (x1, y1, x2, y2) = (x1 as i32, y1 as i32, x2 as i32, y2 as i32);
            let dx = (x2 - x1) as f32;
            let dy = (y2 - y1) as f32;
            let theta = f32::atan2(dy, dx);
            let dx = (x2 - x1) as f32;
            angles.insert(theta);
        }
    }
    Ok(0)
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let parsed = parse(&input)?;

    let some = level1(&parsed)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    // let thing = level2(&parsed)?;
    // writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", some)?;
    Ok(())
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
        let input = parse("asdf")?;
        let result = level1(&input)?;
        assert_eq!(result, ());
        Ok(())
    }
}
