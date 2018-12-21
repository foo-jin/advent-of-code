use std::{
    cmp,
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
    io::{self, Read, Write},
};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<std::error::Error>::from(format!($($tt)*))) }
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Point = [i32; 2];
type Grid = HashMap<Point, HashSet<Point>>;

fn solve() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let (l1, l2) = level(&input)?;
    writeln!(io::stderr(), "level 1: {}", l1)?;
    writeln!(io::stderr(), "level 2: {}", l2)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", l2)?;
    Ok(())
}

fn level(re: &str) -> Result<(u16, usize)> {
    log::info!("building grid");
    let mut grid = Grid::new();
    recursive_descent(&mut grid, re)?;
    // debug_print(&grid)?;

    log::info!("finding maximum distance");
    let result = bfs(&grid);
    Ok(result)
}

fn recursive_descent(grid: &mut Grid, re: &str) -> Result<()> {
    let re = re.trim();
    assert!(re.starts_with('^') && re.ends_with('$'));
    let re = re.trim_matches(|c| c == '^' || c == '$');
    let mut p = [0, 0];
    let mut stack = vec![];

    for c in re.chars() {
        match c {
            '(' => stack.push(p),
            '|' => p = *stack.last().unwrap(),
            ')' => {
                let _ = stack.pop();
            }
            dir => {
                let [x, y] = p;
                let pn = match dir {
                    'N' => [x, y - 1],
                    'S' => [x, y + 1],
                    'E' => [x + 1, y],
                    'W' => [x - 1, y],
                    _ => {
                        log::error!("unknown direction {}", dir);
                        return err!("unknown direction: {}", dir);
                    }
                };
                grid.entry(p).or_default().insert(pn);
                p = pn;
            }
        }
    }

    Ok(())
}

fn bfs(grid: &Grid) -> (u16, usize) {
    const THRESHOLD: u16 = 1000;
    let (mut d_max, mut count) = (0, 0);

    let mut seen = HashSet::new();
    let mut queue = VecDeque::from(vec![([0, 0], 0)]);
    while let Some((p, d)) = queue.pop_front() {
        if !seen.insert(p) {
            continue;
        }

        d_max = cmp::max(d, d_max);
        if d >= THRESHOLD {
            count += 1;
        }

        if let Some(qs) = grid.get(&p) {
            for q in qs {
                queue.push_back((*q, d + 1))
            }
        }
    }

    (d_max, count)
}

fn debug_print(grid: &Grid) -> Result<()> {
    let stderr = io::stderr();
    let mut w = stderr.lock();
    writeln!(w, "Grid:")?;
    let mut points = grid.keys().cloned().collect::<Vec<Point>>();
    points.sort_by_key(|p| (p[1], p[0]));

    let mut lines = [String::new(), String::new()];

    let mut first = true;
    let mut y_prev = 0;
    for p in points {
        log::debug!("{:?}", p);
        let [x, y] = p;
        if first {
            y_prev = y;
            first = false;
        }
        if y != y_prev {
            for l in &mut lines {
                writeln!(w, "{}#", l)?;
                l.clear();
            }
        }
        y_prev = y;

        lines[0].push('#');
        let c = match connected(grid, p, [x - 1, y]) {
            true => '|',
            false => '#',
        };
        lines[1].push(c);

        lines[1].push('.');
        let c = match connected(grid, p, [x, y - 1]) {
            true => '-',
            false => '#',
        };
        lines[0].push(c);
        log::debug!("lines: {:?}", lines);
    }

    let x = lines[1].len();
    for l in &mut lines {
        writeln!(w, "{}#", l)?;
        l.clear();
    }
    writeln!(w, "{}", str::repeat("#", x + 1))?;

    Ok(())
}

fn connected(grid: &Grid, p: Point, q: Point) -> bool {
    let b1 = grid
        .get(&p)
        .map(|neigh| neigh.contains(&q))
        .unwrap_or(false);
    let b2 = grid
        .get(&q)
        .map(|neigh| neigh.contains(&p))
        .unwrap_or(false);
    b1 || b2
}

fn main() -> Result<()> {
    env_logger::init();
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
    const E1: &str = r"^WNE$";
    const E2: &str = r"^ENWWW(NEEE|SSE(EE|N))$";
    const E3: &str = r"^ENNWSWW(NEWS|)SSSEEN(WNSE|)EE(SWEN|)NNN$";

    #[test]
    fn level1_basics() {
        let _ = env_logger::try_init();
        assert_eq!(level(E1).unwrap().0, 3);
        assert_eq!(level(E2).unwrap().0, 10);
        assert_eq!(level(E3).unwrap().0, 18);
    }

    const E4: &str = r"^ESSWWN(E|NNENN(EESS(WNSE|)SSS|WWWSSSSE(SW|NNNE)))$";
    const E5: &str = r"^WSSEESWWWNW(S|NENNEEEENN(ESSSSW(NWSW|SSEN)|WSWWN(E|WWS(E|SS))))$";

    #[test]
    fn level1_advanced() {
        let _ = env_logger::try_init();
        assert_eq!(level(E4).unwrap().0, 23);
        assert_eq!(level(E5).unwrap().0, 31);
    }

    #[test]
    fn regression() {
        let _ = env_logger::try_init();

        assert_eq!(level(INPUT).unwrap(), (3633, 8756))
    }
}
