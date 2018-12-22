use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
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
type Grid = Vec<Vec<TerrainKind>>;

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
    let grid = cave.make_tight_grid();
    grid.iter()
        .flat_map(|row| row.iter())
        .map(|t| *t as u32)
        .sum()
}

fn level2(cave: &Cave) -> u32 {
    let grid = cave.make_grid(1500, 1500);
    astar(cave, &grid)
}

fn astar(cave: &Cave, grid: &Grid) -> u32 {
    use self::Equipment::*;
    use self::TerrainKind::*;

    let init = SearchState::new(cave.target);
    let mut seen = HashSet::new();
    let mut queue = BinaryHeap::new();
    queue.push(init);

    while let Some(ss) = queue.pop() {
        if !seen.insert((ss.pos, ss.equip)) {
            continue;
        }

        if ss.pos == cave.target {
            match ss.equip {
                Torch => return ss.time,
                _ => queue.push(ss.switch_equip(Torch)),
            }
        }

        let [x, y] = ss.pos;
        let current = grid[y][x];
        for (p, terrain) in neighbours(&grid, ss.pos) {
            // note perhaps add transitions for useless stuff too
            let nxt = match (current, ss.equip, terrain) {
                (_, Climbing, Rocky)
                | (_, Torch, Rocky)
                | (_, Climbing, Wet)
                | (_, Neither, Wet)
                | (_, Torch, Narrow)
                | (_, Neither, Narrow) => ss.move_to(p),
                (Wet, Neither, Rocky) | (Rocky, Torch, Wet) => ss.switch_move(Climbing, p),
                (Narrow, Torch, Wet) | (Wet, Climbing, Narrow) => ss.switch_move(Neither, p),
                (Rocky, Climbing, Narrow) | (Narrow, Neither, Rocky) => ss.switch_move(Torch, p),
                _ => panic!(
                    "Invalid state: {:?} terrain with {:?} equipped",
                    current, ss.equip
                ),
            };
            queue.push(nxt);
        }
    }
    unreachable!()
}

fn absdiff(x: usize, y: usize) -> u32 {
    (x as i64 - y as i64).abs() as u32
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SearchState {
    heur: u32,
    time: u32,
    target: Point,
    pos: Point,
    equip: Equipment,
}

impl SearchState {
    fn new(target: Point) -> Self {
        SearchState {
            target,
            ..Self::default()
        }
        .update_heur()
    }

    fn update_heur(mut self) -> Self {
        let [x, y] = self.pos;
        let [tx, ty] = self.target;
        self.heur = self.time + absdiff(tx, x) + absdiff(ty, y);
        self
    }

    fn switch_move(self, eq: Equipment, p: Point) -> Self {
        self.switch_equip(eq).move_to(p)
    }

    fn switch_equip(mut self, eq: Equipment) -> Self {
        if eq != self.equip {
            self.time += 7;
        }
        self.equip = eq;
        self
    }

    fn move_to(mut self, to: Point) -> Self {
        self.time += 1;
        self.pos = to;
        self.update_heur()
    }
}

impl Ord for SearchState {
    fn cmp(&self, other: &SearchState) -> Ordering {
        self.heur.cmp(&other.heur).reverse()
    }
}

impl PartialOrd for SearchState {
    fn partial_cmp(&self, other: &SearchState) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Default for SearchState {
    fn default() -> Self {
        SearchState {
            heur: 0,
            time: 0,
            pos: [0, 0],
            target: [0, 0],
            equip: Equipment::Torch,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Equipment {
    Torch,
    Climbing,
    Neither,
}

#[derive(Clone, Copy, Debug)]
enum TerrainKind {
    Rocky = 0,
    Wet = 1,
    Narrow = 2,
}

impl From<u32> for TerrainKind {
    fn from(k: u32) -> Self {
        use self::TerrainKind::*;
        match k % 3 {
            0 => Rocky,
            1 => Wet,
            2 => Narrow,
            _ => unreachable!(),
        }
    }
}

fn neighbours(grid: &Grid, p: Point) -> impl Iterator<Item = (Point, TerrainKind)> {
    let [x, y] = p;
    grid.get(y + 1)
        .and_then(|row| row.get(x))
        .map(|t| ([x, y + 1], *t))
        .into_iter()
        .chain(
            grid.get(y)
                .and_then(|row| row.get(x + 1))
                .map(|t| ([x + 1, y], *t)),
        )
        .chain(if x > 0 {
            grid.get(y)
                .and_then(|row| row.get(x - 1))
                .map(|t| ([x - 1, y], *t))
        } else {
            None
        })
        .chain(if y > 0 {
            grid.get(y - 1)
                .and_then(|row| row.get(x))
                .map(|t| ([x, y - 1], *t))
        } else {
            None
        })
}

struct Cave {
    target: Point,
    depth: u32,
}

impl Cave {
    fn make_tight_grid(&self) -> Grid {
        let width = self.target[0] as usize + 1;
        let height = self.target[1] as usize + 1;
        self.make_grid(width, height)
    }

    fn make_grid(&self, width: usize, height: usize) -> Grid {
        const X_MULT: u32 = 16807;
        const Y_MULT: u32 = 48271;

        assert!(width > self.target[0]);
        assert!(height > self.target[1]);

        let mut grid = vec![vec![0; width]; height];
        for (x, q) in grid[0].iter_mut().enumerate().skip(1) {
            let geoindex = x as u32 * X_MULT;
            *q = self.erosion(geoindex);
        }

        for y in 1..height {
            let geoindex = y as u32 * Y_MULT;
            grid[y][0] = self.erosion(geoindex);

            for x in 1..width {
                let geoindex = grid[y][x - 1] * grid[y - 1][x];
                grid[y][x] = self.erosion(geoindex);
            }
        }

        let [t_x, t_y] = self.target;
        grid[t_y][t_x] = 0;

        let mut terrain = Vec::with_capacity(height);
        for row in grid {
            let row = row.into_iter().map(|e| TerrainKind::from(e)).collect();
            terrain.push(row);
        }

        terrain
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
