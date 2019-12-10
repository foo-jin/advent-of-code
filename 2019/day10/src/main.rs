use num_rational::Ratio;
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    convert::TryFrom,
    io::{self, Read, Write},
};

type Point = (usize, usize);

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
enum Angle {
    Up,
    Right(Ratio<i32>),
    Down,
    Left(Ratio<i32>),
}

impl Ord for Angle {
    fn cmp(&self, other: &Self) -> Ordering {
        use Angle::*;

        // clockwise order
        match (self, other) {
            (Up, Up) => Ordering::Equal,
            (_, Up) => Ordering::Greater,
            (Up, _) => Ordering::Less,
            (Right(r1), Right(r2)) => r1.cmp(r2),
            (_, Right(_)) => Ordering::Greater,
            (Right(_), _) => Ordering::Less,
            (Left(_), Down) => Ordering::Greater,
            (Down, Down) => Ordering::Equal,
            (Down, Left(_)) => Ordering::Less,
            (Left(r1), Left(r2)) => r1.cmp(r2),
        }
    }
}

impl PartialOrd for Angle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

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

fn levels(meteors: &[Point]) -> aoc::Result<(usize, usize)> {
    use Angle::*;

    let mut max_count = 0;
    let mut location = (0, 0);
    let mut vaporization_plan = BTreeMap::new();
    for &(x1, y1) in meteors {
        log::debug!("============= {:?} =============", (x1, y1));
        let mut angles = BTreeMap::new();
        for &(x2, y2) in meteors {
            if (x2, y2) == (x1, y1) {
                continue;
            }
            let (x1, y1, x2, y2) = (x1 as i32, y1 as i32, x2 as i32, y2 as i32);
            let dx = x2 - x1;
            let dy = y2 - y1;
            let dist = (dx.abs() + dy.abs()) as u32;
            let angle = match (dx, dy) {
                (0, dy) if dy > 0 => Down,
                (0, dy) if dy < 0 => Up,
                (0, _) => unreachable!(),
                (dx, dy) => {
                    let r = Ratio::new(dy, dx);
                    if dx > 0 {
                        Right(r)
                    } else {
                        Left(r)
                    }
                }
            };
            angles.entry(angle).or_insert_with(Vec::new).push((dist, (x2, y2)));
        }
        let count = angles.len();
        log::debug!("count: {}", count);
        if count > max_count {
            max_count = count;
            location = (x1, y1);
            vaporization_plan = angles;
        }
    }

    log::debug!("IMS planted at {:?}", location);

    for (_angle, meteors) in &mut vaporization_plan {
        meteors.sort();
        meteors.reverse();
    }

    let mut destroyed = 0;
    let mut coord = (0, 0);
    loop {
        let mut done = true;
        for (_angle, meteors) in &mut vaporization_plan {
            match meteors.pop() {
                Some((_d, p)) => {
                    done = false;
                    destroyed += 1;
                    if destroyed == 200 {
                        coord = p;
                    }
                    log::debug!("vaporizing astroid #{} at {:?}", destroyed, p);
                }
                None => (),
            }
        }

        if done {
            break;
        }
    }

    let part2 = usize::try_from(coord.0 * 100 + coord.1)?;
    Ok((vaporization_plan.len(), part2))
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let parsed = parse(&input)?;

    let (some, thing) = levels(&parsed)?;
    writeln!(io::stderr(), "level 1: {}", some)?;
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
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
    fn sanity() -> aoc::Result<()> {
        let input = parse(INPUT)?;
        let (p1, p2) = levels(&input)?;
        assert_eq!(p1, 230);
        assert_eq!(p2, 1205);
        Ok(())
    }
}
