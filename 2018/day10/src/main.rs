use std::{
    error::Error,
    io::{self, Read, Write},
    ops, str,
};

#[derive(Clone, Copy, Debug)]
struct Vector {
    dx: i32,
    dy: i32,
}

impl Vector {
    fn new(dx: i32, dy: i32) -> Self {
        Vector { dx, dy }
    }
}

#[derive(Clone, Copy, Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

impl ops::Add<Vector> for Point {
    type Output = Point;

    fn add(self, v: Vector) -> Self::Output {
        Point {
            x: self.x + v.dx,
            y: self.y + v.dy,
        }
    }
}

impl ops::AddAssign<Vector> for Point {
    fn add_assign(&mut self, v: Vector) {
        *self = *self + v;
    }
}

#[derive(Clone, Copy, Debug)]
struct Particle {
    location: Point,
    velocity: Vector,
}

impl str::FromStr for Particle {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use regex::Regex;
        let re = Regex::new(r"([-\d]+)")?;

        let mut nums = Vec::new();
        for c in re.captures_iter(s) {
            let n = c[0].parse::<i32>()?;
            nums.push(n);
        }

        let location = Point::new(nums[0], nums[1]);
        let velocity = Vector::new(nums[2], nums[3]);
        Ok(Particle { location, velocity })
    }
}

impl Particle {
    fn step(&mut self) {
        self.location += self.velocity;
    }
}

fn parse_particles(s: &str) -> Result<Vec<Particle>, Box<dyn Error>> {
    s.lines()
        .map(str::parse::<Particle>)
        .collect::<Result<Vec<Particle>, _>>()
        .map_err(Into::into)
}

fn solve<W: Write>(
    particles: &mut [Particle],
    mut writer: W,
    max_t: u32,
    limit: usize,
) -> Result<u32, Box<dyn Error>> {
    for t in 1..=max_t {
        particles.iter_mut().for_each(Particle::step);

        let upper = particles.iter().map(|p| p.location.y).min().unwrap();
        let lower = particles.iter().map(|p| p.location.y).max().unwrap();
        let left = particles.iter().map(|p| p.location.x).min().unwrap();
        let right = particles.iter().map(|p| p.location.x).max().unwrap();
        let height = (lower - upper) as usize + 1;
        let width = (right - left) as usize + 1;

        if height * width < limit {
            let mut state = vec![vec![false; width]; height];
            for (x, y) in particles
                .iter()
                .map(|p| (p.location.x, p.location.y))
                .map(|(x, y)| ((x - left) as usize, (y - upper) as usize))
                .filter(|(x, y)| *x < width && *y < height)
            {
                state[y][x] = true;
            }

            for line in state.iter().map(|row| {
                row.iter()
                    .map(|b| if *b { "#" } else { "." })
                    .collect::<String>()
            }) {
                writeln!(writer, "{}", line)?;
            }
            return Ok(t);
        }
    }
    Err("Failed to find convergence of particles")?
}

// fn level2(s: &str) -> ... {
//     unimplemented!()
// }

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let mut particles = parse_particles(&input)?;

    let thing = solve(&mut particles, io::stderr().lock(), 100_000, 1000)?;
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");
    const EXAMPLE: &str = "position=< 9,  1> velocity=< 0,  2>
position=< 7,  0> velocity=<-1,  0>
position=< 3, -2> velocity=<-1,  1>
position=< 6, 10> velocity=<-2, -1>
position=< 2, -4> velocity=< 2,  2>
position=<-6, 10> velocity=< 2, -2>
position=< 1,  8> velocity=< 1, -1>
position=< 1,  7> velocity=< 1,  0>
position=<-3, 11> velocity=< 1, -2>
position=< 7,  6> velocity=<-1, -1>
position=<-2,  3> velocity=< 1,  0>
position=<-4,  3> velocity=< 2,  0>
position=<10, -3> velocity=<-1,  1>
position=< 5, 11> velocity=< 1, -2>
position=< 4,  7> velocity=< 0, -1>
position=< 8, -2> velocity=< 0,  1>
position=<15,  0> velocity=<-2,  0>
position=< 1,  6> velocity=< 1,  0>
position=< 8,  9> velocity=< 0, -1>
position=< 3,  3> velocity=<-1,  1>
position=< 0,  5> velocity=< 0, -1>
position=<-2,  2> velocity=< 2,  0>
position=< 5, -2> velocity=< 1,  2>
position=< 1,  4> velocity=< 2,  1>
position=<-2,  7> velocity=< 2, -2>
position=< 3,  6> velocity=<-1, -1>
position=< 5,  0> velocity=< 1,  0>
position=<-6,  0> velocity=< 2,  0>
position=< 5,  9> velocity=< 1, -2>
position=<14,  7> velocity=<-2,  0>
position=<-3,  6> velocity=< 2, -1>";

    #[test]
    fn examples() {
        let mut particles = parse_particles(EXAMPLE).unwrap();
        let t = solve(&mut particles, io::stderr(), 4, 100).unwrap();
        assert_eq!(t, 3);
    }

    #[test]
    fn regression() {
        let mut particles = parse_particles(INPUT).unwrap();
        let t = solve(&mut particles, io::stderr(), 100_000, 1000).unwrap();
        assert_eq!(t, 10681);
    }
}
