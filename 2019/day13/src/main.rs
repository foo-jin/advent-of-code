use intcode::Signal;
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    fmt::{self, Display},
    io::{self, Read, Write},
};

const SCREEN_HEIGHT: usize = 25;
const SCREEN_WIDTH: usize = 40;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tile {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

type Position = (u32, u32);
struct Screen([Tile; SCREEN_WIDTH * SCREEN_HEIGHT]);

impl TryFrom<intcode::Value> for Tile {
    type Error = Box<dyn std::error::Error>;

    fn try_from(v: intcode::Value) -> Result<Self, Self::Error> {
        use Tile::*;
        let tile = match v {
            0 => Empty,
            1 => Wall,
            2 => Block,
            3 => Paddle,
            4 => Ball,
            id => return aoc::err!("Invalid tile id: {}", id),
        };
        Ok(tile)
    }
}

impl Tile {
    fn is_block(&self) -> bool {
        use Tile::*;
        match *self {
            Block => true,
            _ => false,
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Tile::*;
        let c = match &self {
            Empty => ' ',
            Wall => '▓',
            Block => '░',
            Paddle => '=',
            Ball => 'O',
        };
        write!(f, "{}", c)
    }
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", termion::clear::All)?;
        for line in self.0.chunks(SCREEN_HEIGHT) {
            for tile in line {
                write!(f, "{}", tile)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

fn level1(orig_vm: &intcode::VM) -> aoc::Result<usize> {
    let vm = intcode::VM::with_mem(orig_vm.mem());
    let (_tx, rx) = vm.spawn();
    let mut screen = BTreeMap::new();
    loop {
        let x = match rx.recv()? {
            Signal::Value(x) => u32::try_from(x)?,
            Signal::Halting => break,
        };

        let y = match rx.recv()? {
            Signal::Value(y) => u32::try_from(y)?,
            Signal::Halting => return aoc::err!("Unexpected halt"),
        };

        let tile = match rx.recv()? {
            Signal::Value(t) => Tile::try_from(t)?,
            Signal::Halting => return aoc::err!("Unexpected halt"),
        };

        screen.insert((x, y), tile);
    }

    let blocks = screen.values().filter(|t| t.is_block()).count();
    Ok(blocks)
}

fn level2(orig_vm: &intcode::VM) -> aoc::Result<i64> {
    use termion::{clear, cursor};

    let mut vm = intcode::VM::with_mem(orig_vm.mem());
    vm.mem_mut()[0] = 2;
    let (tx, rx) = vm.spawn();

    let mut screen = Screen([Tile::Empty; SCREEN_WIDTH * SCREEN_HEIGHT]);
    let mut ball_x = 21;
    let mut updated = false;
    let mut score = 0;
    let mut paddle_xs = 21;

    eprint!("{}", clear::All);
    loop {
        let x = match rx.recv()? {
            Signal::Value(x) => x,
            Signal::Halting => break,
        };

        let y = match rx.recv()? {
            Signal::Value(y) => usize::try_from(y).unwrap(),
            Signal::Halting => return aoc::err!("Unexpected halt"),
        };

        let value = match rx.recv()? {
            Signal::Value(t) => t,
            Signal::Halting => return aoc::err!("Unexpected halt"),
        };

        if x >= 0 {
            let x = usize::try_from(x)?;
            let tile = Tile::try_from(value)?;
            screen.0[y * SCREEN_HEIGHT + x] = tile;
            log::debug!("{:?}={:?}", (x, y), tile);
            eprint!("{}{}", cursor::Goto(x as u16 + 1, y as u16 + 1), tile);

            if tile == Tile::Ball && x != ball_x {
                ball_x = x;
                let val = if paddle_xs < ball_x {
                    1
                } else if paddle_xs > ball_x {
                    -1
                } else {
                    0
                };
                let _ = tx.send(Signal::Value(val));
                updated = true;
            } else if tile == Tile::Paddle && updated {
                // std::thread::sleep_ms(25);
                paddle_xs = x;
            }
        } else {
            score = value;
            eprint!("{}Score: {}", cursor::Goto(45, 5), score);
            log::debug!("Score: {}", value);
        }

        // eprint!("{}", screen);
    }

    eprintln!("{}", clear::All);
    Ok(score)
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let vm = intcode::VM::with_program(&input)?;

    let some = level1(&vm)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&vm)?;
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
