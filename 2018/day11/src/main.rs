use std::{
    io::{self, Read, Write},
    iter,
};

const GRID_SIZE: usize = 300;

fn power_level(x: u32, y: u32, serial: u32) -> i32 {
    let rack_id = x + 10;
    let mut powlevel = rack_id * y;
    powlevel += serial;
    powlevel *= rack_id;
    powlevel = (powlevel / 100) % 10;
    powlevel as i32 - 5
}

fn max_subgrid(
    serial: u32,
    subgrids: impl IntoIterator<Item = usize>,
) -> (usize, usize, usize) {
    let mut sum = [[0; GRID_SIZE + 1]; GRID_SIZE + 1];
    for y in 1..=GRID_SIZE {
        for x in 1..=GRID_SIZE {
            let val = power_level(x as u32, y as u32, serial);
            let top = sum[y - 1][x];
            let left = sum[y][x - 1];
            let diag = sum[y - 1][x - 1];
            sum[y][x] = val + top + left - diag;
        }
    }

    let mut max_val = None;
    let mut coords = None;
    for s in subgrids {
        for y in s..=GRID_SIZE {
            for x in s..=GRID_SIZE {
                let val = sum[y][x] - sum[y - s][x] - sum[y][x - s]
                    + sum[y - s][x - s];
                if Some(val) > max_val {
                    max_val = Some(val);
                    coords = Some((x - s + 1, y - s + 1, s));
                }
            }
        }
    }
    coords.unwrap()
}

fn level1(serial: u32) -> (usize, usize) {
    let (x, y, _) = max_subgrid(serial, iter::once(3));
    (x, y)
}

fn level2(serial: u32) -> (usize, usize, usize) {
    max_subgrid(serial, 1..=GRID_SIZE)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let serial = input.trim().parse()?;

    let some = level1(serial);
    writeln!(io::stderr(), "level 1: {:?}", some)?;

    let thing = level2(serial);
    writeln!(io::stderr(), "level 2: {:?}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{},{},{}", thing.0, thing.1, thing.2)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test]
    fn power_level_examples() {
        assert_eq!(power_level(3, 5, 8), 4);
        assert_eq!(power_level(122, 79, 57), -5);
        assert_eq!(power_level(217, 196, 39), 0);
        assert_eq!(power_level(101, 153, 71), 4);
    }

    #[test]
    fn level1_examples() {
        assert_eq!(level1(18), (33, 45));
        assert_eq!(level1(42), (21, 61));
    }

    #[test]
    fn level2_examples() {
        assert_eq!(level2(18), (90, 269, 16));
        assert_eq!(level2(42), (232, 251, 12));
    }

    #[test]
    fn level1_regression() {
        let input = INPUT.trim().parse().unwrap();
        assert_eq!(level1(input), (21, 34));
    }

    #[test]
    fn level2_regression() {
        let input = INPUT.trim().parse().unwrap();
        assert_eq!(level2(input), (90, 244, 16));
    }
}
