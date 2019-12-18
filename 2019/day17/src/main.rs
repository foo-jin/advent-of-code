use intcode::Signal;
use num::Integer;
use std::{
    collections::HashSet as Set,
    convert::TryFrom,
    fmt,
    io::{self, Read, Write},
};

#[derive(Clone, Copy, Debug)]
struct Position(usize, usize, Direction);

struct View {
    data: Vec<u8>,
    width: usize,
    height: usize,
    start_position: Position,
    scaffold_len: usize,
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Clone, Copy, Debug)]
enum Turn {
    Right,
    Left,
    Straight(usize),
}

impl View {
    fn read_map(vm: intcode::VM) -> aoc::Result<Self> {
        let (_tx, rx) = vm.spawn();
        let mut view = Vec::new();
        loop {
            match rx.recv()? {
                Signal::Value(v) => view.push(u8::try_from(v)?),
                Signal::Halting => break,
            }
        }

        let width = view.iter().position(|b| *b == b'\n').ok_or_else(|| {
            aoc::format_err!("Expected input to consist of multiple lines")
        })? + 1;
        let height = view.len() / width;

        use Direction::*;

        let mut init_position = None;
        let mut scaffold_len = 0;
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let i = y * width + x;
                scaffold_len += 1;
                match view[i] {
                    b'#' =>
                        if view[i - width] == b'#'
                            && view[i + width] == b'#'
                            && view[i - 1] == b'#'
                            && view[i + 1] == b'#'
                        {
                            view[i] = b'O';
                        },
                    b'^' => init_position = Some(Position(x, y, North)),
                    b'v' => init_position = Some(Position(x, y, South)),
                    b'>' => init_position = Some(Position(x, y, East)),
                    b'<' => init_position = Some(Position(x, y, West)),
                    _ => scaffold_len -= 1,
                }
                if view[i] == b'#'
                    && view[i - width] == b'#'
                    && view[i + width] == b'#'
                    && view[i - 1] == b'#'
                    && view[i + 1] == b'#'
                {
                    view[i] = b'O';
                }
            }
        }

        let start_position = init_position.ok_or_else(|| {
            "Expected input to contain initial position for cleaner robot"
        })?;
        Ok(View { data: view, width, height, start_position, scaffold_len })
    }
}

impl Position {
    fn apply(self, turn: Turn) -> Self {
        use Direction::*;
        use Turn::*;
        let Position(x, y, d) = self;
        let (x, y, d) = match (d, turn) {
            (North, Left) => (x, y, West),
            (South, Left) => (x, y, East),
            (West, Left) => (x, y, South),
            (East, Left) => (x, y, North),
            (North, Right) => (x, y, East),
            (South, Right) => (x, y, West),
            (West, Right) => (x, y, North),
            (East, Right) => (x, y, South),
            (North, Straight(dy)) => (x, y.wrapping_sub(dy), d),
            (South, Straight(dy)) => (x, y.checked_add(dy).unwrap(), d),
            (West, Straight(dx)) => (x.wrapping_sub(dx), y, d),
            (East, Straight(dx)) => (x.checked_add(dx).unwrap(), y, d),
        };
        Position(x, y, d)
    }
}

struct Path<'a>(&'a [Turn]);
impl<'a> fmt::Display for Path<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Turn::*;
        let mut first = true;
        for t in self.0 {
            if first {
                first = false;
            } else {
                write!(f, ",")?
            }

            match t {
                Right => write!(f, "R"),
                Left => write!(f, "L"),
                Straight(x) => write!(f, "{}", x),
            }?;
        }
        Ok(())
    }
}

fn level1(view: &View) -> usize {
    view.data
        .iter()
        .enumerate()
        .filter(|(_i, &x)| x == b'O')
        .map(|(i, _x)| {
            let (d, r) = i.div_rem(&view.width);
            d * r
        })
        .sum()
}

fn level2(view: &View, mut vm: intcode::VM) -> aoc::Result<intcode::Value> {
    use Turn::*;
    let mut path = vec![];
    let mut pos = view.start_position;

    'outer: loop {
        let new_pos = pos.apply(Straight(1));
        let Position(x, y, _d) = new_pos;
        if x < view.width - 1
            && y < view.height
            && is_scaffold(view.data[y * view.width + x])
        {
            if let Some(Straight(d1)) = path.last() {
                *path.last_mut().unwrap() = Straight(d1 + 1);
                pos = new_pos;
            }
            continue 'outer;
        }

        for &t in &[Right, Left] {
            let new_pos = pos.apply(t).apply(Straight(1));
            let Position(x, y, _d) = new_pos;
            if x < view.width - 1
                && y < view.height
                && is_scaffold(view.data[y * view.width + x])
            {
                pos = new_pos;
                path.push(t);
                path.push(Straight(1));
                continue 'outer;
            }
        }
        break;
    }
    let path = Path(&path).to_string();
    eprintln!("{}", path);

    // R,10,R,10,R,6,R,4,R,10,R,10,L,4,R,10,R,10,R,6,R,4,R,4,L,4,L,10,L,10,R,10,R,10,R,6,R,4,R,10,R,10,L,4,R,4,L,4,L,10,L,10,R,10,R,10,L,4,R,4,L,4,L,10,L,10,R,10,R,10,L,4
    // apply this perl-style regex somewhere else:
    // ^(.{1,21})\1*(.{1,21})(?:\1|\2)*(.{1,21})(?:\1|\2|\3)*$
    // The capture groups are the subroutines.

    let a = &path[..17];
    let b = &path[18..31];
    let c = &path[50..67];
    let main = path.replace(a, "A").replace(b, "B").replace(c, "C");
    vm.mem_mut()[0] = 2;

    let (tx, rx) = vm.spawn();
    for part in &[&main, a, b, c] {
        eprintln!("{}", part);
        for c in part.bytes() {
            eprint!("{} ", c);
            let sig = Signal::Value(c as i64);
            tx.send(sig).unwrap();
        }
        tx.send(Signal::Value(10)).unwrap();
        eprintln!("10");
    }
    tx.send(Signal::Value(i64::from(b'n'))).unwrap();
    tx.send(Signal::Value(10)).unwrap();

    for msg in rx.recv() {
        match msg {
            Signal::Value(v) => eprint!("{}", u8::try_from(v).unwrap() as char),
            Signal::Halting =>
                return aoc::err!("Program halted before giving output"),
        };
    }

    Ok(0)
}

fn is_scaffold(b: u8) -> bool {
    match b {
        b'#' | b'>' | b'<' | b'^' | b'v' | b'O' => true,
        _ => false,
    }
}

fn id_dfs(
    view: &View,
    seen: &mut Set<usize>,
    pos: Position,
    depth: usize,
) -> Option<Vec<Turn>> {
    use Turn::*;
    let Position(x, y, d) = pos;
    if depth == 0 || x >= view.width - 1 || y >= view.height {
        return None;
    }

    let i = y * view.width + x;
    match view.data[i] {
        b'#' | b'O' | b'<' | b'>' | b'^' | b'v' => (),
        _ => return None,
    }

    let modified = seen.insert(i);
    if seen.len() == view.scaffold_len {
        return Some(Vec::new());
    }
    for &t in &[Right, Left, Straight(1)] {
        let new_pos = pos.apply(t);
        if let Some(mut path) = id_dfs(view, seen, new_pos, depth - 1) {
            if let Some(Straight(d1)) = path.last() {
                if let Straight(d2) = t {
                    *path.last_mut().unwrap() = Straight(d1 + d2);
                    return Some(path);
                }
            }

            path.push(t);
            return Some(path);
        }
    }

    if modified {
        seen.remove(&i);
    }

    None
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let vm = intcode::VM::with_program(&input)?;
    let view = View::read_map(intcode::VM::with_mem(vm.mem()))?;

    let some = level1(&view);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&view, vm)?;
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
        let input = parse("asdf")?;
        let result = level1(&input)?;
        assert_eq!(result, ());
        Ok(())
    }
}
