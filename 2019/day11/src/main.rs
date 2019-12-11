use intcode::Signal;
use std::{
    cmp::Reverse,
    collections::{BTreeSet, HashSet},
    io::{self, Read, Write},
};

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn right(self) -> Self {
        use Direction::*;
        match self {
            Up => Right,
            Right => Down,
            Down => Left,
            Left => Up,
        }
    }

    fn left(self) -> Self {
        use Direction::*;
        match self {
            Right => Up,
            Down => Right,
            Left => Down,
            Up => Left,
        }
    }
}

fn level1(orig_vm: &intcode::VM) -> aoc::Result<usize> {
    use Direction::*;

    let vm = intcode::VM::with_mem(orig_vm.mem());
    let (tx, rx) = vm.spawn();
    let mut canvas = HashSet::new();
    let mut white = HashSet::new();
    let mut direction = Up;
    let mut p = (0i32, 0i32);
    let painted = loop {
        let (x, y) = p;
        let colour = if white.contains(&(x, y)) { 1 } else { 0 };
        let _ = tx.send(Signal::Value(colour));
        match rx.recv()? {
            Signal::Halting => break canvas.len(),
            Signal::Value(0) => {
                white.remove(&(x, y));
            },
            Signal::Value(1) => {
                white.insert((x, y));
            },
            Signal::Value(z) =>
                return aoc::err!("Unknown paint instruction received: {}", z),
        }

        canvas.insert((x, y));
        direction = match rx.recv()? {
            Signal::Halting => break canvas.len(),
            Signal::Value(0) => direction.left(),
            Signal::Value(1) => direction.right(),
            Signal::Value(z) =>
                return aoc::err!("Unknown turn instruction received: {}", z),
        };

        p = match direction {
            Up => (x, y + 1),
            Right => (x + 1, y),
            Down => (x, y - 1),
            Left => (x - 1, y),
        }
    };

    Ok(painted)
}

fn level2(orig_vm: &intcode::VM) -> aoc::Result<()> {
    use Direction::*;

    let vm = intcode::VM::with_mem(orig_vm.mem());
    let (tx, rx) = vm.spawn();
    let mut canvas = BTreeSet::new();
    let mut white = HashSet::new();
    white.insert((0, 0));
    let mut direction = Up;
    let mut p = (0i32, 0i32);
    let _ = loop {
        let (x, y) = p;
        let colour = if white.contains(&(y, x)) { 1 } else { 0 };
        let _ = tx.send(Signal::Value(colour));
        match rx.recv()? {
            Signal::Halting => break canvas.len(),
            Signal::Value(0) => {
                white.remove(&(y, x));
            },
            Signal::Value(1) => {
                white.insert((y, x));
            },
            Signal::Value(z) =>
                return aoc::err!("Unknown paint instruction received: {}", z),
        }

        canvas.insert((Reverse(y), x));
        direction = match rx.recv()? {
            Signal::Halting => break canvas.len(),
            Signal::Value(0) => direction.left(),
            Signal::Value(1) => direction.right(),
            Signal::Value(z) =>
                return aoc::err!("Unknown turn instruction received: {}", z),
        };

        p = match direction {
            Up => (x, y + 1),
            Right => (x + 1, y),
            Down => (x, y - 1),
            Left => (x - 1, y),
        }
    };

    let mut left_outline = 0;
    let mut right_outline = 0;
    let mut top = 0;
    let mut bottom = 0;
    for &(y, x) in &canvas {
        left_outline = i32::min(x, left_outline);
        right_outline = i32::max(x, right_outline);
        top = i32::max(y.0, top);
        bottom = i32::min(y.0, bottom);
    }

    let mut xpos = left_outline;
    let mut ypos = top;
    for &(y, x) in &canvas {
        let y = y.0;
        let paint = if white.contains(&(y, x)) { '▓' } else { '░' };
        while y < ypos {
            for _ in xpos..=right_outline {
                eprint!(" ")
            }
            eprint!("\n");
            xpos = left_outline;
            ypos -= 1;
        }
        while xpos != x {
            eprint!(" ");
            xpos += 1;
        }
        eprint!("{}", paint);
        xpos += 1;
    }
    eprint!("\n");

    Ok(())
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let vm = intcode::VM::with_program(&input)?;

    let some = level1(&vm)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    writeln!(io::stderr(), "level 2:")?;
    let _ = level2(&vm)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", some)?;
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
        Ok(())
    }
}
