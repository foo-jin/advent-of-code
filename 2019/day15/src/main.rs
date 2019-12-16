use intcode::Signal;
use std::{
    collections::{HashMap as Map, HashSet as Set},
    convert::TryFrom,
    io::{self, Read, Write},
    sync::mpsc,
};

const CARDINAL: [Direction; 4] =
    [Direction::North, Direction::East, Direction::South, Direction::West];
const ORIGIN: Position = Position(0, 0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Terrain {
    Wall,
    Empty,
    Oxygen,
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Position(i32, i32);

impl TryFrom<intcode::Value> for Terrain {
    type Error = Box<dyn std::error::Error>;

    fn try_from(val: intcode::Value) -> Result<Self, Self::Error> {
        let l = match val {
            0 => Terrain::Wall,
            1 => Terrain::Empty,
            2 => Terrain::Oxygen,
            v => return aoc::err!("Invalid status code: {}", v),
        };
        Ok(l)
    }
}

impl Into<intcode::Value> for Direction {
    fn into(self) -> intcode::Value {
        use Direction::*;
        match self {
            North => 1,
            South => 2,
            West => 3,
            East => 4,
        }
    }
}

impl Position {
    fn neighbours(self) -> impl Iterator<Item = Self> {
        CARDINAL.iter().map(move |&d| self.apply(d))
    }

    fn apply(self, d: Direction) -> Self {
        use Direction::*;
        let mut x = self.0;
        let mut y = self.1;
        match d {
            North => y += 1,
            South => y -= 1,
            East => x += 1,
            West => x -= 1,
        }
        Position(x, y)
    }
}

impl Direction {
    fn reverse(self) -> Self {
        use Direction::*;
        match self {
            North => South,
            South => North,
            East => West,
            West => East,
        }
    }
}

fn level1(map: &Map<Position, Terrain>) -> aoc::Result<u32> {
    bfs(map, ORIGIN, |mapstate, pos| mapstate[&pos] == Terrain::Oxygen, |_| ())
        .ok_or_else(|| aoc::format_err!("Failed to find oxygen on the map"))
}

fn level2(map: &Map<Position, Terrain>) -> u32 {
    let start = map
        .iter()
        .find(|&(_, &t)| t == Terrain::Oxygen)
        .map(|(p, _)| p)
        .cloned()
        .unwrap();
    bfs(
        map,
        start,
        |map, _p| !map.values().any(|&terrain| terrain == Terrain::Empty),
        |t| match *t {
            Terrain::Empty => *t = Terrain::Oxygen,
            _ => (),
        },
    )
    .expect("Failed flood-fill")
}

fn bfs<F, G>(
    map: &Map<Position, Terrain>,
    p_init: Position,
    goal_reached: F,
    update: G,
) -> Option<u32>
where
    F: Fn(&Map<Position, Terrain>, Position) -> bool,
    G: Fn(&mut Terrain),
{
    let mut map = map.clone();
    let mut time = 0;
    let mut outer: Vec<Position> = p_init.neighbours().collect();
    let mut seen = Set::new();
    loop {
        time += 1;
        let mut next = Vec::new();
        if outer.is_empty() {
            return None;
        }
        for p in outer {
            if !seen.insert(p)
                || !map.contains_key(&p)
                || map[&p] == Terrain::Wall
            {
                continue;
            }

            let t = map.get_mut(&p).unwrap();
            update(t);

            if goal_reached(&map, p) {
                return Some(time);
            }

            next.extend(p.neighbours());
        }
        outer = next;
    }
}

fn explore_area(vm: intcode::VM) -> aoc::Result<Map<Position, Terrain>> {
    fn dfs(
        map: &mut Map<Position, Terrain>,
        current_pos: Position,
        tx: &mpsc::Sender<Signal>,
        rx: &mpsc::Receiver<Signal>,
    ) -> aoc::Result<()> {
        use Terrain::*;

        let terrain = map[&current_pos];
        match terrain {
            Wall => Ok(()),
            Empty | Oxygen => {
                for &d in CARDINAL.iter() {
                    let next_pos = current_pos.apply(d);
                    if map.contains_key(&next_pos) {
                        continue;
                    }
                    tx.send(Signal::Value(d.into())).unwrap();
                    let resp = rx.recv().unwrap();
                    let terrain = extract_status_code(resp)?;
                    map.insert(next_pos, terrain);

                    if terrain != Wall {
                        dfs(map, next_pos, tx, rx)?;
                        tx.send(Signal::Value(d.reverse().into())).unwrap();
                        let resp = rx.recv().unwrap();
                        let terrain = extract_status_code(resp)?;
                        assert_ne!(terrain, Wall);
                    }
                }

                Ok(())
            },
        }
    }

    let (tx, rx) = vm.spawn();
    let mut map = Map::new();
    map.insert(ORIGIN, Terrain::Empty);
    dfs(&mut map, ORIGIN, &tx, &rx)?;
    let _ = tx.send(Signal::Halting);
    Ok(map)
}

fn extract_status_code(sig: Signal) -> aoc::Result<Terrain> {
    let v = match sig {
        Signal::Value(v) => v,
        Signal::Halting => panic!("unexpected halt"),
    };
    Terrain::try_from(v)
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let vm = intcode::VM::with_program(&input)?;
    let map = explore_area(vm)?;

    let some = level1(&map)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&map);
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
        let vm = intcode::VM::with_program(INPUT)?;
        let map = explore_area(vm)?;
        let some = level1(&map)?;
        assert_eq!(some, 208, "part 1");

        let thing = level2(&map);
        assert_eq!(thing, 306, "part 2");
        Ok(())
    }
}
