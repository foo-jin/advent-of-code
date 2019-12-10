use std::io::{self, Read, Write};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<std::error::Error>::from(format!($($tt)*))) }
}

macro_rules! format_err {
    ($($tt:tt)*) => { Box::<std::error::Error>::from(format!($($tt)*)) }
}

mod aoc {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}

type Point = (i32, i32);

#[derive(PartialEq, Eq, Copy, Clone)]
enum Orientation {
    Vertical,
    Horizontal,
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let some = level1(&input);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&input);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

fn level1(s: &str) -> u64 {
    use Orientation::*;

    let mut wires = s.trim().lines().map(|s| {
        let mut pos = (0, 0);
        let mut trails = Vec::new();
        for turn in s.split(',') {
            let (x, y) = pos;
            let (direction, distance) = turn.split_at(1);
            let distance = distance.parse::<i32>().unwrap();
            let (or, new_pos) = match direction {
                "U" => (Vertical, (x, y + distance)),
                "D" => (Vertical, (x, y - distance)),
                "R" => (Horizontal, (x + distance, y)),
                "L" => (Horizontal, (x - distance, y)),
                _ => panic!("Unknown direction encountered: {}", direction),
            };
            trails.push((or, pos, new_pos));
            pos = new_pos;
        }
        trails
    });
    let a = wires.next().unwrap();
    let b = wires.next().unwrap();

    let mut manhattan = std::u64::MAX;
    for (oa, pa1, pa2) in &a {
        for (ob, pb1, pb2) in &b {
            let (x, x1, x2, y, y1, y2) = match (oa, ob) {
                (Horizontal, Vertical) =>
                    (pb1.0, pa1.0, pa2.0, pa1.1, pb1.1, pb2.1),
                (Vertical, Horizontal) =>
                    (pa1.0, pb1.0, pb2.0, pb1.1, pa1.1, pa2.1),
                _ => continue,
            };

            let (x1, x2) = (i32::min(x1, x2), i32::max(x1, x2));
            let (y1, y2) = (i32::min(y1, y2), i32::max(y1, y2));
            if x1 < x && x < x2 && y1 < y && y < y2 {
                manhattan = u64::min(manhattan, (x.abs() + y.abs()) as u64);
            }
        }
    }
    manhattan
}

fn level2(s: &str) -> u64 {
    use Orientation::*;

    let mut wires = s.trim().lines().map(|s| {
        let mut pos = (0, 0);
        let mut trails = Vec::new();
        let mut len = 0;
        for turn in s.split(',') {
            let (x, y) = pos;
            let (direction, distance) = turn.split_at(1);
            let distance = distance.parse::<i32>().unwrap();
            let (or, new_pos) = match direction {
                d @ "U" => (d, (x, y + distance)),
                d @ "D" => (d, (x, y - distance)),
                d @ "R" => (d, (x + distance, y)),
                d @ "L" => (d, (x - distance, y)),
                _ => panic!("Unknown direction encountered: {}", direction),
            };
            trails.push((or, pos, new_pos, len));
            pos = new_pos;
            len += distance as u64;
        }
        trails
    });
    let a = wires.next().unwrap();
    let b = wires.next().unwrap();

    let mut steps = std::u64::MAX;
    for (da, pa1, pa2, la) in &a {
        for (db, pb1, pb2, lb) in &b {
            let oa =
                if *da == "U" || *da == "D" { Vertical } else { Horizontal };

            let ob =
                if *db == "U" || *db == "D" { Vertical } else { Horizontal };

            let (x, x1, x2, y, y1, y2) = match (oa, ob) {
                (Horizontal, Vertical) =>
                    (pb1.0, pa1.0, pa2.0, pa1.1, pb1.1, pb2.1),
                (Vertical, Horizontal) =>
                    (pa1.0, pb1.0, pb2.0, pb1.1, pa1.1, pa2.1),
                _ => continue,
            };

            let (x1, x2) = (i32::min(x1, x2), i32::max(x1, x2));
            let (y1, y2) = (i32::min(y1, y2), i32::max(y1, y2));
            if x1 < x && x < x2 && y1 < y && y < y2 {
                let mut st = la + lb;
                st += match *da {
                    "U" => y - y1,
                    "D" => y2 - y,
                    "L" => x2 - x,
                    "R" => x - x1,
                    _ => panic!("agh"),
                } as u64;
                st += match *db {
                    "U" => y - y1,
                    "D" => y2 - y,
                    "L" => x2 - x,
                    "R" => x - x1,
                    _ => panic!("agh"),
                } as u64;
                steps = u64::min(steps, st);
            }
        }
    }
    steps
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
    fn level1_examples() {
        let input = "R8,U5,L5,D3\n\
                     U7,R6,D4,L4";
        assert_eq!(level1(input), 6);

        let input = "R75,D30,R83,U83,L12,D49,R71,U7,L72\n\
                     U62,R66,U55,R34,D71,R55,D58,R83";
        assert_eq!(level1(input), 159);

        let input = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\n\
                     U98,R91,D20,R16,D67,R40,U7,R15,U6,R7";
        assert_eq!(level1(input), 135);
    }

    #[test]
    fn level2_examples() {
        let input = "R8,U5,L5,D3\n\
                     U7,R6,D4,L4";
        assert_eq!(level2(input), 30);

        let input = "R75,D30,R83,U83,L12,D49,R71,U7,L72\n\
                     U62,R66,U55,R34,D71,R55,D58,R83";
        assert_eq!(level2(input), 610);

        let input = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\n\
                     U98,R91,D20,R16,D67,R40,U7,R15,U6,R7";
        assert_eq!(level2(input), 410);
    }
}
