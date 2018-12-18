use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    io::{self, Read, Write},
    ops,
};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Open,
    Trees,
    Lumberyard,
}

fn parse_grid(s: &str) -> Result<Box<[Box<[State]>]>, Box<dyn Error>> {
    let mut grid = Vec::new();
    for l in s.trim().lines() {
        let mut line = Vec::with_capacity(l.len());
        for c in l.chars() {
            let state = match c {
                '.' => State::Open,
                '|' => State::Trees,
                '#' => State::Lumberyard,
                c => return err!("unexpected character in input: {}", c),
            };
            line.push(state);
        }
        grid.push(line.into_boxed_slice());
    }

    Ok(grid.into_boxed_slice())
}

fn debug_print(grid: &[Box<[State]>]) {
    let stderr = io::stderr();
    let mut w = stderr.lock();
    for row in grid {
        for s in row.iter() {
            let c = match s {
                State::Open => '.',
                State::Trees => '|',
                State::Lumberyard => '#',
            };
            write!(w, "{}", c).unwrap();
        }
        write!(w, "\n").unwrap();
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Neighbours {
    open: u8,
    trees: u8,
    lumber: u8,
}

impl ops::AddAssign<State> for Neighbours {
    fn add_assign(&mut self, rhs: State) {
        match rhs {
            State::Open => self.open += 1,
            State::Trees => self.trees += 1,
            State::Lumberyard => self.lumber += 1,
        }
    }
}

fn neighbours(grid: &[Box<[State]>], x: usize, y: usize) -> Neighbours {
    let mut neighbours = Neighbours::default();
    let left = x > 0;
    let right = x + 1 < grid[0].len();
    let up = y > 0;
    let down = y + 1 < grid.len();

    if left {
        let x = x - 1;
        neighbours += grid[y][x];
        if up {
            neighbours += grid[y - 1][x];
        }
        if down {
            neighbours += grid[y + 1][x];
        }
    }
    if right {
        let x = x + 1;
        neighbours += grid[y][x];
        if up {
            neighbours += grid[y - 1][x];
        }
        if down {
            neighbours += grid[y + 1][x];
        }
    }
    if up {
        neighbours += grid[y - 1][x];
    }
    if down {
        neighbours += grid[y + 1][x];
    }
    neighbours
}

fn simulate(mut grid: Box<[Box<[State]>]>, t_max: u32) -> u32 {
    const LIMIT: u8 = 3;

    let stderr = io::stderr();
    let mut w = stderr.lock();

    let (mut wood, mut lumber) = (0, 0);
    for s in grid.iter().flat_map(|row| row.iter()) {
        match s {
            State::Trees => wood += 1,
            State::Lumberyard => lumber += 1,
            _ => (),
        }
    }

    let mut history = HashMap::new();
    for t in 1..=t_max {
        let mut next = grid.clone();
        for y in 0..grid.len() {
            for x in 0..grid[0].len() {
                let s = grid[y][x];
                let cs = neighbours(&grid, x, y);
                next[y][x] = match s {
                    State::Open if cs.trees > 2 => {
                        wood += 1;
                        State::Trees
                    }
                    State::Trees if cs.lumber > 2 => {
                        wood -= 1;
                        lumber += 1;
                        State::Lumberyard
                    }
                    State::Lumberyard if cs.lumber < 1 || cs.trees < 1 => {
                        lumber -= 1;
                        State::Open
                    }
                    s => s,
                };
            }
        }
        grid = next;

        let score = wood * lumber;
        writeln!(w, "t: {}, pos: {}, score: {}", t, t % 28, score).unwrap();
        match history.entry(score) {
            Entry::Vacant(e) => {
                e.insert((t, 1));
            }
            Entry::Occupied(mut e) => {
                let (t_0, count) = e.get_mut();
                if *count < LIMIT {
                    *count += 1;
                    *t_0 = t;
                } else {
                    let t_0 = *t_0;
                    let cycle_len = t - t_0;
                    let t_result = t_0 + (t_max % cycle_len) + 1;
                    writeln!(
                        w,
                        "t0: {}\ncycle: {}\nt_result: {}",
                        t_0, cycle_len, t_result
                    )
                    .unwrap();
                    return history
                        .into_iter()
                        .find(|(_score, (t, _))| *t == t_result)
                        .map(|(score, _t)| score)
                        .unwrap();
                }
            }
        }
    }

    wood * lumber
}

fn level1(grid: Box<[Box<[State]>]>) -> u32 {
    simulate(grid, 10)
}

fn level2(grid: Box<[Box<[State]>]>) -> u32 {
    simulate(grid, 1_000_000_000)
}

fn solve() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let grid = parse_grid(&input)?;

    let some = level1(grid.clone());
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(grid);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
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
    const EXAMPLE: &str = "
.#.#...|#.
.....#|##|
.|..|...#.
..|#.....#
#.#|||#|#|
...#.||...
.|....|...
||...#|.#|
|.||||..|.
...#.|..|.";

    #[test]
    fn level1_examples() {
        let grid = parse_grid(EXAMPLE).unwrap();
        assert_eq!(level1(grid), 1147);
    }

    #[test]
    fn level1_regression() {
        let grid = parse_grid(INPUT).unwrap();
        assert_eq!(level1(grid), 495236);
    }

    #[test]
    fn level2_regression() {
        let grid = parse_grid(INPUT).unwrap();
        assert_eq!(level2(grid), 201348);
    }
}
