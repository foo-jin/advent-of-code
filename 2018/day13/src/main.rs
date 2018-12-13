use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::{self, Read, Write},
    ops, str,
};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Point = (usize, usize);

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl ops::Add<Turn> for Direction {
    type Output = Self;

    fn add(self, turn: Turn) -> Self::Output {
        use self::Direction::*;
        match (self, turn) {
            (dir, Turn::Straight) => dir,
            (Up, Turn::Left) => Left,
            (Up, Turn::Right) => Right,
            (Down, Turn::Left) => Right,
            (Down, Turn::Right) => Left,
            (Left, Turn::Left) => Down,
            (Left, Turn::Right) => Up,
            (Right, Turn::Left) => Up,
            (Right, Turn::Right) => Down,
        }
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
        let (x, y) = self.pos;
        let new = match self.facing {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        };
        self.pos = new;
    }

    fn turn(&mut self, node: Node) {
        use self::Direction::*;
        let turn = match node {
            Node::Intersection => {
                let turn = self.turn[0];
                self.turn.rotate_left(1);
                turn
            }
            Node::LRTurn => match self.facing {
                Up | Down => Turn::Right,
                Left | Right => Turn::Left,
            },
            Node::RLTurn => match self.facing {
                Up | Down => Turn::Left,
                Left | Right => Turn::Right,
            },
        };
        self.facing += turn;
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
                        carts.push(Cart::new(id, (x, y), dir));
                        continue;
                    }
                    ' ' | '-' | '|' => continue,
                    c => return err!("unexpected character while parsing graph: {}", c),
                };
                nodes.insert((x, y), node);
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
            let (x, y) = cart.pos;
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

fn level1(mut graph: Graph) -> (usize, usize) {
    loop {
        if let Some(collision) = graph.tick() {
            return collision;
        }
    }
}

fn level2(mut graph: Graph) -> (usize, usize) {
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
    write!(io::stdout(), "{},{}", thing.0, thing.1)?;

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
        assert_eq!(level1(graph), (7, 3))
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
        assert_eq!(level2(graph), (6, 4))
    }

    #[test]
    fn level1_regression() {
        let graph = INPUT.parse().unwrap();
        assert_eq!(level1(graph), (58, 93))
    }

    #[test]
    fn level2_regression() {
        let graph = INPUT.parse().unwrap();
        assert_eq!(level2(graph), (91, 72))
    }
}
