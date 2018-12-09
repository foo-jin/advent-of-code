#![feature(slice_patterns)]
use std::{
    io::{self, Read, Write},
    num,
};

fn parse_license(s: &str) -> Result<Vec<usize>, num::ParseIntError> {
    s.trim().split_whitespace().map(str::parse).collect()
}

fn level1(license: &[usize]) -> usize {
    fn process_node(license: &[usize]) -> (&[usize], usize) {
        match license {
            [] | [_] => panic!("shouldn't happen?"),
            [children, meta, ..] => {
                let children = *children;
                let meta = *meta;
                let license = &license[2..];

                let (license, mut sum) = (0..children).fold((license, 0), |(lic, sum), _| {
                    let (rest, val) = process_node(lic);
                    (rest, sum + val)
                });

                let (meta, license) = license.split_at(meta);
                sum += meta.iter().sum::<usize>();
                (license, sum)
            }
        }
    }

    let (_rest, value) = process_node(license);
    value
}

fn level2(license: &[usize]) -> usize {
    fn process_node(license: &[usize]) -> (&[usize], usize) {
        match license {
            [] | [_] => panic!("Invariant violated: license is a slice of length < 2"),
            [children, meta, ..] => {
                let children = *children;
                let meta = *meta;
                let license = &license[2..];

                let mut license = license;
                let mut vals = vec![0; children];
                for val in vals.iter_mut() {
                    let (rest, v) = process_node(license);
                    license = rest;
                    *val = v;
                }

                let (meta, license) = license.split_at(meta);
                let sum = match children {
                    0 => meta.iter().sum::<usize>(),
                    _ => meta
                        .iter()
                        .filter_map(|m| vals.get(m.wrapping_sub(1)))
                        .sum::<usize>(),
                };

                (license, sum)
            }
        }
    }

    let (_rest, value) = process_node(license);
    value
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let license = parse_license(&input)?;

    let some = level1(&license);
    writeln!(io::stderr(), "level 1: {}", some)?;

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
