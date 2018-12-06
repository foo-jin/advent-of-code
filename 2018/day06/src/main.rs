use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::{self, Read, Write},
    num,
};

const LIMIT: u32 = 10000;

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

fn manhattan(p: Point, q: Point) -> u32 {
    ((p.0 - q.0).abs() + (p.1 - q.1).abs()) as u32
}

fn neighbours((px, py): Point) -> impl Iterator<Item = Point> {
    use std::iter;
    let left = iter::once((px - 1, py));
    let right = iter::once((px + 1, py));
    let up = iter::once((px, py - 1));
    let down = iter::once((px, py + 1));
    left.chain(right).chain(up).chain(down)
}

trait IterExt: Iterator {
    fn min_by_key_unique<B: Ord, F>(mut self, mut f: F) -> Option<Self::Item>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> B,
    {
        self.next()
            .map(|first| {
                let first_p = f(&first);

                self.fold((first_p, first, false), |(sel_p, sel, duplicate), x| {
                    let x_p = f(&x);
                    if x_p < sel_p {
                        (x_p, x, false)
                    } else if x_p == sel_p {
                        (sel_p, sel, true)
                    } else {
                        (sel_p, sel, duplicate)
                    }
                })
            })
            .and_then(|(_sel_p, sel, duplicate)| if !duplicate { Some(sel) } else { None })
    }
}

impl<T: Iterator> IterExt for T {}

fn level1(points: &[Point]) -> usize {
    let upper = points.iter().map(|(_x, y)| *y).min().unwrap();
    let lower = points.iter().map(|(_x, y)| *y).max().unwrap();
    let left = points.iter().map(|(x, _y)| *x).min().unwrap();
    let right = points.iter().map(|(x, _y)| *x).max().unwrap();
    let mut infinite = HashSet::new();
    let mut counts = HashMap::new();

    for y in upper..=lower {
        for x in left..=right {
            let p = (x, y);
            if let Some(closest) = points
                .iter()
                .map(|q| (q, manhattan(p, *q)))
                .min_by_key_unique(|(_q, d)| *d)
                .map(|(closest, _d)| closest)
            {
                if infinite.contains(&closest) {
                    continue;
                }

                if x == left || x == right || y == upper || y == lower {
                    infinite.insert(closest);
                    counts.remove(&closest);
                } else {
                    *counts.entry(closest).or_insert(0) += 1;
                }
            }
        }
    }

    // counts.retain(|k, _v| !infinite.contains(k));
    *counts.values().max().unwrap()
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

    #[test]
    fn level1_regression() {
        let input = parse_points(INPUT).unwrap();
        assert_eq!(level1(&input), 5941);
    }

    #[test]
    fn level2_regression() {
        let input = parse_points(INPUT).unwrap();
        assert_eq!(level2(&input, LIMIT), 40244);
    }
}
