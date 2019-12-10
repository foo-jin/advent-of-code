use petgraph::prelude as pg;
use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    str::FromStr,
};
use vpsearch as vps;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<std::error::Error>::from(format!($($tt)*))) }
}

mod aoc {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let points = parse_points(&input)?;

    let (graph, some) = level1(&points);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let mut dot = File::create("graph.gv")?;
    write!(dot, "{:?}", petgraph::dot::Dot::new(&graph))?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", some)?;
    Ok(())
}

fn level1(points: &[Point]) -> (Graph, usize) {
    log::debug!("{:?}", points);
    let vp = vps::Tree::new(points);
    let mut graph = Graph::new();
    for p in points {
        graph.add_node(*p);
        let connected =
            vp.find_nearest_custom(p, &(), ConstellationSearch::default());
        for (q, dist) in connected.into_iter().filter(|(q, _)| q != p) {
            assert!(dist <= LIMIT);
            graph.add_edge(*p, q, dist);
        }
    }
    assert_eq!(points.len(), graph.node_count());
    let components = petgraph::algo::connected_components(&graph);
    (graph, components)
}

type Graph = pg::UnGraphMap<Point, u8>;
const LIMIT: u8 = 3;

fn absdiff(a: i8, b: i8) -> u8 {
    (a - b).abs() as u8
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Point([i8; 4]);

impl vps::MetricSpace for Point {
    type Distance = u8;
    type UserData = ();

    fn distance(&self, other: &Point, _: &Self::UserData) -> Self::Distance {
        self.0.iter().zip(other.0.iter()).map(|(a, b)| absdiff(*a, *b)).sum()
    }
}

#[derive(Default)]
struct ConstellationSearch {
    result: <Self as vps::BestCandidate<Point, ()>>::Output,
}

impl vps::BestCandidate<Point, ()> for ConstellationSearch {
    type Output = Vec<(Point, u8)>;

    fn consider(&mut self, p: &Point, dist: u8, _idx: usize, _: &()) {
        if dist <= LIMIT {
            self.result.push((*p, dist));
        }
    }

    fn distance(&self) -> u8 {
        LIMIT + 1
    }

    fn result(self, _: &()) -> Self::Output {
        self.result
    }
}

impl FromStr for Point {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        log::trace!("Point::from_str({})", s);
        use regex::Regex;
        lazy_static::lazy_static! {
            static ref RE: Regex = Regex::new(r"([-]?[0-9]+)").unwrap();
        }
        let mut nums = [0; 4];
        let mut changes = 0;
        for (i, k) in RE.captures_iter(s.trim()).enumerate() {
            log::trace!("cap {}: {}", i, &k[0]);
            nums[i] = k[0].parse()?;
            changes += 1;
        }

        if changes < 4 {
            err!("not enough coordinates given")
        } else {
            Ok(Point(nums))
        }
    }
}

fn parse_points(s: &str) -> aoc::Result<Vec<Point>> {
    s.trim().lines().map(str::parse).collect()
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

    fn check_level1(s: &str, expected: usize, msg: &str) -> aoc::Result<()> {
        let points = parse_points(s)?;
        assert_eq!(level1(&points).1, expected, "{}", msg);
        Ok(())
    }

    #[test_log::new]
    fn level1_examples() -> aoc::Result<()> {
        check_level1(E1, 2, "E1")?;
        check_level1(E2, 4, "E2")?;
        check_level1(E3, 3, "E3")?;
        check_level1(E4, 8, "E4")?;
        Ok(())
    }

    const E1: &str = "
0,0,0,0
 3,0,0,0
 0,3,0,0
 0,0,3,0
 0,0,0,3
 0,0,0,6
 9,0,0,0
12,0,0,0";

    const E2: &str = "
-1,2,2,0
0,0,2,-2
0,0,0,-2
-1,2,0,0
-2,-2,-2,2
3,0,2,-1
-1,3,2,2
-1,0,-1,0
0,2,1,-2
3,0,0,0";

    const E3: &str = "
1,-1,0,1
2,0,-1,0
3,2,-1,0
0,0,3,1
0,0,-1,-1
2,3,-2,0
-2,2,0,0
2,-2,0,-1
1,-1,0,-1
3,2,0,2";

    const E4: &str = "
1,-1,-1,-2
-2,-2,0,1
0,2,1,3
-2,3,-2,1
0,2,3,-2
-1,-1,1,-2
0,-2,-1,0
-2,2,3,-1
1,2,2,0
-1,-2,0,-2";
}
