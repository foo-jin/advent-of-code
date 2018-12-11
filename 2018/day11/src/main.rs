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

fn level1(serial: u32) -> (usize, usize) {
    let mut grid = [[0; GRID_SIZE]; GRID_SIZE];
    grid.iter_mut()
        .enumerate()
        .flat_map(|(y, row)| iter::repeat(y).zip(row.iter_mut().enumerate()))
        .map(|(y, (x, val))| (y as u32, x as u32, val))
        .for_each(|(y, x, val)| *val = power_level(x, y, serial));

    let mut max_val = None;
    let mut coords = None;
    for y in 0..GRID_SIZE - 3 {
        for x in 0..GRID_SIZE - 3 {
            let mut val = 0;
            for i in 0..3 {
                for j in 0..3 {
                    val += grid[y + i][x + j];
                }
            }
            if Some(val) > max_val {
                max_val = Some(val);
                coords = Some((x, y));
            }
        }
    }
    coords.unwrap()
}

fn level2(serial: u32) -> (usize, usize, usize) {
    let mut grid = [[0; GRID_SIZE]; GRID_SIZE];
    grid.iter_mut()
        .enumerate()
        .flat_map(|(y, row)| iter::repeat(y).zip(row.iter_mut().enumerate()))
        .map(|(y, (x, val))| (y as u32, x as u32, val))
        .for_each(|(y, x, val)| *val = power_level(x, y, serial));

    let mut max_val = None;
    let mut coords = None;

    for chunk_size in 0..GRID_SIZE {
        for y in 0..GRID_SIZE - chunk_size {
            for x in 0..GRID_SIZE - chunk_size {
                let mut val = 0;
                for i in 0..chunk_size {
                    for j in 0..chunk_size {
                        val += grid[y + i][x + j];
                    }
                }
                if Some(val) > max_val {
                    max_val = Some(val);
                    coords = Some((x, y, chunk_size));
                }
            }
        }
    }
    coords.unwrap()
}
// fn level2(s: &str) -> ... {
//     unimplemented!()
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let serial = input.trim().parse()?;

    // let some = level1(serial);
    // writeln!(io::stderr(), "level 1: {:?}", some)?;

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

    // #[test]
    // fn level1_regression() {
    //     assert_eq!(level1(INPUT), 6150);
    // }
}
