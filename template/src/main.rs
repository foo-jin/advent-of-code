use std::{
    io::{self, Read},
};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let parsed = parse(&input);

    let some = level1(&parsed);
    eprintln!("level 1: {}", some);

    let thing = level2(&parsed);
    eprintln!("level 2: {}", thing);

    // stdout is used to submit solutions
    println!("{}", some);
}

fn parse(s: &str) -> u32 {
    0
}

fn level1(input: &u32) -> u32 {
    0
}

fn level2(input: &u32) -> u32 {
    0
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test]
    fn level1_examples() {
        assert_eq!(1, 1)
    }

    // #[test]
    fn level2_examples() {
        assert_eq!(1, 1)
    }

    // #[test]
    fn level1_sanity() {
        assert_eq!(1, 1)
    }

    // #[test]
    fn level2_sanity() {
        assert_eq!(1, 1)
    }
}
