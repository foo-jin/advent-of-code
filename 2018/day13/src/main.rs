#![feature(try_from)]

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::{self, Read, Write},
    ops, str,
};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Point = [u16; 2];

#[derive(Clone, Copy, Debug, PartialEq)]
struct Direction([i8; 2]);

#[allow(non_upper_case_globals)]
impl Direction {
    const Up: Direction = Direction([0, -1]);
    const Down: Direction = Direction([0, 1]);
    const Left: Direction = Direction([-1, 0]);
    const Right: Direction = Direction([1, 0]);

    fn lr_turn(&mut self) {
        let [x, y] = self.0;
        self.0 = [-y, -x];
    }

    fn rl_turn(&mut self) {
        let [x, y] = self.0;
        self.0 = [y, x];
    }
}

impl ops::Add<Turn> for Direction {
    type Output = Self;

    fn add(self, turn: Turn) -> Self::Output {
        let dir = match (self.0, turn) {
            (dir, Turn::Straight) => dir,
            ([x, y], Turn::Left) => [y, -x],
            ([x, y], Turn::Right) => [-y, x],
        };
        Direction(dir)
    }
}

impl ops::AddAssign<Turn> for Direction {
    fn add_assign(&mut self, turn: Turn) {
        *self = *self + turn;
    }
}
#[derive(Clone, Copy, Debug)]
enum Turn {
    Straight,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
struct Cart {
    id: usize,
    pos: Point,
    facing: Direction,
    turn: [Turn; 3],
}

impl Cart {
    fn new(id: usize, pos: Point, facing: Direction) -> Self {
        let turn = [Turn::Left, Turn::Straight, Turn::Right];
        Cart {
            id,
            pos,
            facing,
            turn,
        }
    }

    fn update(&mut self) {
        use std::convert::TryFrom;
        for (x, dx) in self.pos.iter_mut().zip(self.facing.0.iter()) {
            *x = u16::try_from(i32::from(*x) + i32::from(*dx)).expect("cart went out of bounds");
        }
    }

    fn turn(&mut self, node: Node) {
        match node {
            Node::Intersection => {
                let turn = self.turn[0];
                self.turn.rotate_left(1);
                self.facing += turn;
            }
            Node::LRTurn => self.facing.lr_turn(),
            Node::RLTurn => self.facing.rl_turn(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Node {
    Intersection,
    LRTurn,
    RLTurn,
}

#[derive(Clone, Debug)]
struct Graph {
    nodes: HashMap<Point, Node>,
    carts: Vec<Cart>,
    positions: HashSet<Point>,
}

impl str::FromStr for Graph {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut nodes = HashMap::new();
        let mut carts = Vec::new();
        for (y, l) in s.lines().enumerate() {
            for (x, c) in l.chars().enumerate() {
                let pos = [x as u16, y as u16];
                let id = carts.len();
                let node = match c {
                    '/' => Node::LRTurn,
                    '\\' => Node::RLTurn,
                    '+' => Node::Intersection,
                    '>' | '<' | '^' | 'v' => {
                        let dir = match c {
                            '>' => Direction::Right,
                            '<' => Direction::Left,
                            '^' => Direction::Up,
                            'v' => Direction::Down,
                            _ => unreachable!(),
                        };
                        carts.push(Cart::new(id, pos, dir));
                        continue;
                    }
                    ' ' | '-' | '|' => continue,
                    c => return err!("unexpected character while parsing graph: {}", c),
                };
                nodes.insert(pos, node);
            }
        }
        let positions = carts.iter().map(|c| c.pos).collect();
        let mut graph = Graph {
            nodes,
            carts,
            positions,
        };
        graph.sort_carts();
        Ok(graph)
    }
}

impl Graph {
    fn sort_carts(&mut self) {
        self.carts.sort_unstable_by_key(|cart| {
            let [x, y] = cart.pos;
            (y, x)
        });
    }

    fn tick(&mut self) -> Option<Point> {
        let mut collisions = HashSet::new();
        let mut first = None;
        for cart in self.carts.iter_mut() {
            if collisions.contains(&cart.pos) {
                continue;
            }

            self.positions.remove(&cart.pos);
            cart.update();
            if !self.positions.insert(cart.pos) {
                first = first.or_else(|| Some(cart.pos));
                collisions.insert(cart.pos);
                self.positions.remove(&cart.pos);
            }

            if let Some(node) = self.nodes.get(&cart.pos) {
                cart.turn(*node);
            }
        }

        self.carts = self
            .carts
            .iter()
            .cloned()
            .filter(|c| !collisions.contains(&c.pos))
            .collect();
        self.sort_carts();

        first
    }
}

fn level1(mut graph: Graph) -> Point {
    loop {
        if let Some(collision) = graph.tick() {
            return collision;
        }
    }
}

fn level2(mut graph: Graph) -> Point {
    while graph.carts.len() > 1 {
        let _ = graph.tick();
    }

    graph.carts[0].pos
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let graph = input.parse::<Graph>()?;

    let some = level1(graph.clone());
    writeln!(io::stderr(), "level 1: {:?}", some)?;

    let thing = level2(graph);
    writeln!(io::stderr(), "level 2: {:?}", thing)?;

    // stdout is used to submit solutions
    write!(io::stdout(), "{},{}", thing[0], thing[1])?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");
    const EXAMPLE1: &str = r"/->-\        
|   |  /----\
| /-+--+-\  |
| | |  | v  |
\-+-/  \-+--/
  \------/   ";

    #[test]
    fn level1_examples() {
        let graph = EXAMPLE1.parse().unwrap();
        assert_eq!(level1(graph), [7, 3])
    }

    const EXAMPLE2: &str = r"/>-<\  
|   |  
| /<+-\
| | | v
\>+</ |
  |   ^
  \<->/";

    #[test]
    fn level2_examples() {
        let graph = EXAMPLE2.parse().unwrap();
        assert_eq!(level2(graph), [6, 4])
    }

    #[test]
    fn level1_regression() {
        let graph = INPUT.parse().unwrap();
        assert_eq!(level1(graph), [58, 93])
    }

    #[test]
    fn level2_regression() {
        let graph = INPUT.parse().unwrap();
        assert_eq!(level2(graph), [91, 72])
    }
}
