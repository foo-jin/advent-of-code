use std::{
    collections::{VecDeque, HashMap, HashSet},
    io::{self, Read, Write},
    num,
};

type Point = (i32, i32);

fn parse_points(s: &str) -> Result<Vec<Point>, num::ParseIntError> {
    s.trim()
        .lines()
        .map(|s| {
            let mut parts = s.split(", ");
            let x = parts.next().expect("no x coordinate found");
            let y = parts.next().expect("no y coordinate found");
            x.parse().and_then(|x| y.parse().map(|y| (x, y)))
        })
        .collect()
}

#[allow(dead_code)]
fn print_grid(grid: &HashMap<Point, State>) {
    use itertools::Itertools;

    let mut cells = grid.iter().collect::<Vec<_>>();
    cells.sort_unstable_by_key(|(p, _s)| p.0);
    cells.sort_by_key(|(p, _s)| p.1);

    for (_k, group) in &cells.into_iter().group_by(|(p, _s)| p.1) {
        eprint!("{}", group.map(|c| c.1).join("|"));
        eprintln!()
    }
    eprintln!()
}

enum State {
    Claim(Point),
    Owned(Point),
    Neutral,
    Empty,
}
use std::fmt;

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            State::Claim(p) | State::Owned(p) => write!(f, "{}{}", p.0, p.1),
            State::Neutral => write!(f, ".."),
            State::Empty => write!(f, "--"),
        }
    }
}

fn neighbours((px, py): Point) -> impl Iterator<Item = Point> {
    use std::iter;
    let left = iter::once((px - 1, py));
    let right = iter::once((px + 1, py));
    let up = iter::once((px, py - 1));
    let down = iter::once((px, py + 1));
    left.chain(right).chain(up).chain(down)
}

fn level1(points: &[Point]) -> usize {
    use maplit::hashset;

    let mut grid = HashMap::new();
    let mut sizes: HashMap<Point, usize> = HashMap::new();
    let mut flood = points
        .iter()
        .map(|&p| (p, hashset! {p}))
        .collect::<Vec<(Point, HashSet<Point>)>>();
    let mut infinite = HashSet::new();

    let upper = points.iter().map(|(_x, y)| *y).min().unwrap();
    let lower = points.iter().map(|(_x, y)| *y).max().unwrap();
    let left = points.iter().map(|(x, _y)| *x).min().unwrap();
    let right = points.iter().map(|(x, _y)| *x).max().unwrap();

    while !flood.iter().all(|(_, b)| b.is_empty()) {
        flood = flood
            .into_iter()
            .map(|(p, bound)| {
                let mut new_bound = HashSet::new();
                for q in bound {
                    let e = grid.entry(q).or_insert(State::Empty);
                    match e {
                        State::Empty => *e = State::Claim(p),
                        State::Claim(_) => {*e = State::Neutral; continue },
                        _ => continue,
                    }
                    let (x, y) = q;
                    if !(x < left || x > right || y < upper || y > lower) {
                        new_bound.extend(neighbours(q));
                    } else {
                        infinite.insert(p);
                    }
                }
                (p, new_bound)
            })
            .collect();

        for (_p, state) in grid.iter_mut() {
            match *state {
                State::Claim(q) => {
                    *state = State::Owned(q);
                    *sizes.entry(q).or_insert(0) += 1;
                }
                _ => (),
            }
        }
    }

    sizes
        .iter()
        .filter(|(p, _v)| !infinite.contains(*p))
        .map(|(_p, v)| *v)
        .max()
        .unwrap()
}

fn manhattan(p: Point, q: Point) -> u32 {
    ((p.0 - q.0).abs() + (p.1 - q.1).abs()) as u32
}

fn level2(points: &[Point], limit: u32) -> usize {
    let mut queue = points.iter().cloned().collect::<VecDeque<Point>>();
    let mut seen = HashSet::new();
    let mut size = 0;
    while let Some(p) = queue.pop_front() {
        if !seen.insert(p) {
            continue;
        }

        let total = points.iter().map(|q| manhattan(p, *q)).sum::<u32>();
        if total < limit {
            size += 1;
            queue.extend(neighbours(p));
        }
    }
    size
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let points = parse_points(&input)?;

    let some = level1(&points);
    writeln!(io::stderr(), "level 1: {}", some)?;

    const LIMIT: u32 = 10000;
    let thing = level2(&points, LIMIT);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");
    const EXAMPLE: &str = "1, 1
1, 6
8, 3
3, 4
5, 5
8, 9";

    #[test]
    fn level1_examples() {
        let input = parse_points(EXAMPLE).unwrap();
        assert_eq!(level1(&input), 17)
    }

    #[test]
    fn level2_examples() {
        let input = parse_points(EXAMPLE).unwrap();
        assert_eq!(level2(&input, 32), 16)
    }

    // #[test]
    // fn level1_regression() {
    //     assert_eq!(level1(INPUT), 6150);
    // }

    // #[test]
    // fn level2_regression() {
    //     assert_eq!(level2(INPUT), Some("rteotyxzbodglnpkudawhijsc".to_string()));
    // }
}
