use intcode::Signal;
use std::{
    collections::{HashMap as Map, HashSet as Set},
    convert::TryFrom,
    io::{self, Read, Write},
    sync::mpsc,
};

const CARDINAL: [Direction; 4] =
    [Direction::North, Direction::East, Direction::South, Direction::West];

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

fn level1(vm: intcode::VM) -> aoc::Result<u32> {
    use Terrain::*;

    let mut mapped = Map::new();
    let mut current_pos = Position(0, 0);
    mapped.insert(current_pos, Empty);
    let mut unknown =
        CARDINAL.iter().cloned().map(|d| (current_pos, d)).collect::<Vec<_>>();
    let (tx, rx) = vm.spawn();
    while let Some((known_pos, direction)) = unknown.pop() {
        let target = known_pos.apply(direction);
        if mapped.contains_key(&target) {
            continue;
        }
        // dbg!(&mapped, current_pos, known_pos, direction);
        for d in find_path(&mapped, current_pos, known_pos).unwrap() {
            tx.send(Signal::Value(d.into())).unwrap();
            match extract_status_code(rx.recv().unwrap())? {
                Empty => current_pos = current_pos.apply(d),
                Oxygen => return aoc::err!("Oxygen found, wtf are we doing"),
                Wall => panic!("wall hit while following found path"),
            }
        }

        tx.send(Signal::Value(direction.into())).unwrap();
        let next = current_pos.apply(direction);
        let terrain = extract_status_code(rx.recv().unwrap())?;
        // dbg!(current_pos);
        // dbg!(next, terrain);
        mapped.insert(next, terrain);
        match terrain {
            Empty => {
                current_pos = next;
                assert_ne!(mapped[&current_pos], Wall);
                unknown
                    .extend(CARDINAL.iter().cloned().map(|d| (current_pos, d)));
            },
            Oxygen => {
                let path = find_path(&mapped, Position(0, 0), next)
                    .expect("Could not find path to reached position");
                let _ = tx.send(Signal::Halting);
                return Ok(path.len() as u32);
            },
            Wall => (),
        }
    }
    aoc::err!("Did not find oxygen")
}

fn level2(vm: intcode::VM) -> aoc::Result<u32> {
    use Terrain::*;

    let mut map = explore_area(vm)?;
    let mut time = 0;
    let start = map.iter().find(|&(_p, &t)| t == Oxygen).unwrap();
    let mut outer = vec![*start.0];
    loop {
        if !map.values().any(|&terrain| terrain == Empty) {
            break;
        }

        let mut next = Vec::new();
        for p in outer {
            for q in CARDINAL.iter().map(|&d| p.apply(d)) {
                let current = map.get_mut(&q).unwrap();
                match current {
                    Wall | Oxygen => (),
                    Empty => {
                        *current = Oxygen;
                        next.push(q);
                    },
                }
            }
        }
        outer = next;
        time += 1;
    }
    Ok(time)
}

fn extract_status_code(sig: Signal) -> aoc::Result<Terrain> {
    let v = match sig {
        Signal::Value(v) => v,
        Signal::Halting => panic!("unexpected halt"),
    };
    Terrain::try_from(v)
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
    map.insert(Position(0, 0), Terrain::Empty);
    dfs(&mut map, Position(0, 0), &tx, &rx)?;
    let _ = tx.send(Signal::Halting);
    Ok(map)
}

fn find_path(
    map: &Map<Position, Terrain>,
    from: Position,
    to: Position,
) -> Option<Vec<Direction>> {
    fn dfs(
        map: &Map<Position, Terrain>,
        from: Position,
        to: Position,
        seen: &mut Set<Position>,
    ) -> Option<Vec<Direction>> {
        use Terrain::*;
        if !seen.insert(from) {
            return None;
        }
        if let Some(&terrain) = map.get(&from) {
            if terrain == Empty || terrain == Oxygen {
                if from == to {
                    return Some(Vec::new());
                }

                for d in CARDINAL.iter().cloned() {
                    let next = from.apply(d);
                    if let Some(mut path) = dfs(map, next, to, seen) {
                        path.push(d);
                        return Some(path);
                    }
                }
            }
        }
        seen.remove(&from);
        None
    }

    // dbg!(map, from ,to);
    // eprintln!("==================");

    let mut seen = Set::new();
    dfs(map, from, to, &mut seen).map(|mut p| {
        p.reverse();
        p
    })
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let vm = intcode::VM::with_program(&input)?;

    // let some = level1(vm)?;
    // writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(vm)?;
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
    fn level1_examples() -> aoc::Result<()> {
        let vm = intcode::VM::with_program("asdf")?;
        let result = level1(&input)?;
        assert_eq!(result, ());
        Ok(())
    }
}
