use pathfinding::prelude as pf;
use std::{
    cmp,
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
type Point = [usize; 2];

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let cave = input.parse()?;

    let some = level1(&cave);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&cave);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

fn level1(cave: &Cave) -> u32 {
    let [x, y] = cave.target;
    let grid = cave.make_grid(x, y);
    grid.to_vec().into_iter().sum()
}

fn level2(cave: &Cave) -> u32 {
    const Z: usize = 2;
    let [tx, ty] = cave.target;
    let width = cmp::max(500, tx * Z);
    let height = cmp::max(500, ty * Z);
    let grid = cave.make_grid(width, height);
    let (_, t) = pf::astar(
        &((0, 0), TORCH),
        |&(p, eq)| {
            grid.neighbours(&p, false)
                .filter(|&q| ALLOWED[grid[&q] as usize] & eq == eq)
                .map(|(nx, ny)| (((nx, ny), eq), 1))
                .chain(std::iter::once(((p, ALLOWED[grid[&p] as usize] - eq), 7)))
                .collect::<Vec<_>>()
        },
        |&((x, y), _)| pf::absdiff(x, tx) + pf::absdiff(y, ty),
        |&state| state == ((tx, ty), TORCH),
    )
    .expect("failed to find target");
    t as u32
}

const NEITHER: usize = 1;
const TORCH: usize = 2;
const GEAR: usize = 4;

const ALLOWED: [usize; 3] = [TORCH + GEAR, NEITHER + GEAR, NEITHER + TORCH];

struct Cave {
    target: Point,
    depth: u32,
}

impl Cave {
    fn make_grid(&self, width: usize, height: usize) -> pf::Matrix<u32> {
        const FX: u32 = 16807;
        const FY: u32 = 48271;

        let mut grid = pf::Matrix::new(width + 1, height + 1, 0);
        for y in 0..=height {
            for x in 0..=width {
                let p = (x, y);
                let geoindex = match p {
                    (0, 0) => 0,
                    (_, 0) => x as u32 * FX,
                    (0, _) => y as u32 * FY,
                    (x, y) => grid[&(x - 1, y)] * grid[&(x, y - 1)],
                };
                grid[&p] = self.erosion(geoindex);
            }
        }

        let [t_x, t_y] = self.target;
        grid[&(t_x, t_y)] = 0;
        grid.as_mut().iter_mut().for_each(|n| *n = *n % 3);
        grid
    }

    fn erosion(&self, geoindex: u32) -> u32 {
        const EROSION_MOD: u32 = 20183;
        (geoindex + self.depth) % EROSION_MOD
    }
}

impl FromStr for Cave {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(
            r"(?x)
                depth:\s(?P<depth>[0-9]+)\s
                target:\s(?P<x>[0-9]+),(?P<y>[0-9]+)",
        )?;
        let caps = re
            .captures(s)
            .ok_or_else(|| format_err!("unexpected input format"))?;
        let depth = caps["depth"].parse()?;
        let p = [caps["x"].parse()?, caps["y"].parse()?];
        let cave = Cave { target: p, depth };
        Ok(cave)
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
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");
    const EX: &str = "depth: 510
target: 10,10";

    #[test_log::new]
    fn level1_examples() -> aoc::Result<()> {
        let cave = EX.parse()?;
        assert_eq!(level1(&cave), 114);
        Ok(())
    }

    #[test_log::new]
    fn level2_examples() -> aoc::Result<()> {
        let cave = EX.parse()?;
        assert_eq!(level2(&cave), 45);
        Ok(())
    }

    #[test_log::new]
    fn level1_regression() -> aoc::Result<()> {
        let cave = INPUT.parse()?;
        assert_eq!(level1(&cave), 4479);
        Ok(())
    }

    #[test_log::new]
    fn level2_regression() -> aoc::Result<()> {
        let cave = INPUT.parse()?;
        assert_eq!(level2(&cave), 1032);
        Ok(())
    }
}
