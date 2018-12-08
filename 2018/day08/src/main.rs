#![feature(slice_patterns)]
use std::{
    io::{self, Read, Write},
    num,
};

fn parse_license(s: &str) -> Result<Vec<usize>, num::ParseIntError> {
    s.trim().split_whitespace().map(str::parse).collect()
}

fn level1(mut license: &[usize]) -> usize {
    fn process_node(license: &mut &[usize]) -> usize {
        match license {
            [] | [_] => panic!("shouldn't happen?"),
            [children, meta, ..] => {
                let children = *children;
                let meta = *meta;
                *license = &license[2..];

                let mut sum = 0;
                for _ in 0..children {
                    sum += process_node(license);
                }

                sum += license[..meta].iter().sum::<usize>();
                *license = &license[meta..];
                sum
            }
        }
    }

    process_node(&mut license)
}

fn level2(mut license: &[usize]) -> usize {
    fn process_node_index(license: &mut &[usize]) -> usize {
        match license {
            [] | [_] => panic!("shouldn't happen?"),
            [children, meta, ..] => {
                let children = *children;
                let meta = *meta;
                *license = &license[2..];

                let mut sums = vec![0; children];
                for i in 0..children {
                    sums[i] += process_node_index(license);
                }

                let sum = match children {
                    0 => license[..meta].iter().sum::<usize>(),
                    _ => license[..meta]
                        .iter()
                        .filter_map(|m| sums.get(m.wrapping_sub(1)))
                        .sum::<usize>(),
                };

                *license = &license[meta..];
                sum
            }
        }
    }

    process_node_index(&mut license)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let license = parse_license(&input)?;

    // let some = level1(&license);
    // writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&license);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");
    const EXAMPLE: &str = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2";

    #[test]
    fn level1_examples() {
        let input = parse_license(EXAMPLE).unwrap();
        assert_eq!(level1(&input), 138)
    }

    #[test]
    fn level2_examples() {
        let input = parse_license(EXAMPLE).unwrap();
        assert_eq!(level2(&input), 66)
    }

    #[test]
    fn level1_regression() {
        let input = parse_license(INPUT).unwrap();
        assert_eq!(level1(&input), 45868);
    }

    #[test]
    fn level2_regression() {
        let input = parse_license(INPUT).unwrap();
        assert_eq!(level2(&input), 19724);
    }
}
