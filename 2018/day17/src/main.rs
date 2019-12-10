use std::{
    cmp,
    collections::{hash_map::Entry, HashMap},
    error::Error,
    fmt,
    io::{self, Read, Write},
    str,
};

use self::GroundState::*;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Point = [u16; 2];

#[derive(Clone, Copy, Debug, PartialEq)]
enum GroundState {
    Clay,
    Flowing,
    Settled,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Action {
    Fill,
    Done,
    Settle,
}

impl Action {
    fn is_fill(&self) -> bool {
        match self {
            Action::Fill => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
struct Scan {
    grid: HashMap<Point, GroundState>,
    y_min: u16,
    y_max: u16,
    water: u32,
}

impl fmt::Display for Scan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut grid = vec![vec![' '; 212]; self.y_max as usize + 1];
        let dx = 500 - 106;
        let mid = 500 - dx;
        grid[0][mid] = '+';
        for ([x, y], state) in &self.grid {
            let x = *x as usize;
            let y = *y as usize;
            let c = match state {
                Flowing => '|',
                Settled => '~',
                Clay => '#',
            };
            if x > dx || x - dx - 1 < grid[0].len() {
                grid[y][x - dx - 1] = c;
            }
        }

        for line in grid {
            let line = line.into_iter().collect::<String>();
            writeln!(f, "{}", line)?;
        }

        Ok(())
    }
}

impl str::FromStr for Scan {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(r"([-\d]+)")?;
        let mut grid = HashMap::new();
        let mut y_min = None;
        let mut y_max = None;
        for l in s.lines() {
            let (fixed, rest) = l.split_at(1);
            let mut nums = Vec::new();
            for c in re.captures_iter(rest) {
                let n = c[0].parse::<u16>()?;
                nums.push(n);
            }
            let xy = nums[0];
            let lo = nums[1];
            let hi = nums[2];
            for line in lo..=hi {
                let p = match fixed {
                    "x" => [xy, line],
                    "y" => [line, xy],
                    _ => return err!("Invalid fixed axis: {}", fixed),
                };
                grid.insert(p, GroundState::Clay);
                let max = y_max.get_or_insert(p[1]);
                *max = cmp::max(*max, p[1]);
                let min = y_min.get_or_insert(p[1]);
                *min = cmp::min(*min, p[1]);
            }
        }

        let scan = Scan {
            grid,
            y_min: y_min.unwrap(),
            y_max: y_max.unwrap(),
            water: 0,
        };

        Ok(scan)
    }
}

impl Scan {
    fn update_water(&mut self, p: Point, t: GroundState) {
        match self.grid.entry(p) {
            Entry::Occupied(mut e) => {
                e.insert(t);
            },
            Entry::Vacant(e) => {
                e.insert(t);
                if p[1] >= self.y_min {
                    self.water += 1;
                }
            },
        }
    }

    fn settle(&mut self, p: Point) {
        self.update_water(p, GroundState::Settled);
    }

    fn flow(&mut self, p: Point) {
        self.update_water(p, GroundState::Flowing);
    }

    fn flood(&mut self, p: Point) -> Action {
        match self.grid.get(&p) {
            Some(Flowing) | Some(Settled) => panic!("nani"),
            Some(Clay) => Action::Settle,
            None => {
                self.flow(p);
                let [x, y] = p;
                return self.waterfall([x, y + 1]);
            },
        }
    }

    fn waterfall(&mut self, p: Point) -> Action {
        let [x, y] = p;
        if y > self.y_max {
            return Action::Done;
        }

        match self.grid.get(&p) {
            Some(Clay) | Some(Settled) => return Action::Fill,
            Some(Flowing) => return Action::Done,
            None => (),
        }

        self.flow(p);

        match self.waterfall([x, y + 1]) {
            Action::Settle => panic!("nani!"),
            Action::Done => Action::Done,
            Action::Fill => {
                let (mut left, mut l) = (Action::Fill, 0);
                while left.is_fill() {
                    l += 1;
                    left = self.flood([x - l, y]);
                }

                let (mut right, mut r) = (Action::Fill, 0);
                while right.is_fill() {
                    r += 1;
                    right = self.flood([x + r, y]);
                }

                match (left, right) {
                    (Action::Settle, Action::Settle) => {
                        let l = x - (l - 1);
                        let r = x + (r - 1);
                        (l..=r).for_each(|x| self.settle([x, y]));
                        Action::Fill
                    },
                    _ => Action::Done,
                }
            },
        }
    }
}

fn level1(s: &str) -> Result<u32, Box<dyn Error>> {
    let mut scan = s.parse::<Scan>()?;
    scan.waterfall([500, 1]);
    eprintln!("{}", scan);
    Ok(scan.water)
}

fn level2(s: &str) -> Result<u32, Box<dyn Error>> {
    let mut scan = s.parse::<Scan>()?;
    scan.waterfall([500, 1]);
    let c = scan.grid.values().filter(|s| **s == GroundState::Settled).count();
    Ok(c as u32)
}

fn solve() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let some = level1(&input)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&input)?;
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
    const EXAMPLE: &str = "x=495, y=2..7
y=7, x=495..501
x=501, y=3..7
x=498, y=2..4
x=506, y=1..2
x=498, y=10..13
x=504, y=10..13
y=13, x=498..504";

    #[test]
    fn level1_examples() {
        assert_eq!(level1(EXAMPLE).unwrap(), 57);
    }

    #[test]
    fn level2_examples() {
        assert_eq!(level2(EXAMPLE).unwrap(), 29);
    }

    #[test]
    fn level1_regression() {
        assert_eq!(level1(INPUT).unwrap(), 31883);
    }

    #[test]
    fn level2_regression() {
        assert_eq!(level2(INPUT).unwrap(), 24927);
    }
}
