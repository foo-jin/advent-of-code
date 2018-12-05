use std::io::{self, Read, Write};

fn opposites(a: char, b: char) -> bool {
    a != b && a.to_ascii_uppercase() == b.to_ascii_uppercase()
}

fn full_reaction(chars: impl Iterator<Item = char>) -> usize {
    let mut remaining = Vec::new();

    for a in chars {
        if let Some(b) = remaining.pop() {
            if opposites(a, b) {
                continue;
            }
            remaining.push(b);
        }
        remaining.push(a);
    }

    remaining.len()
}

fn level1(s: &str) -> usize {
    full_reaction(s.trim().chars())
}

fn level2(s: &str) -> usize {
    let chars = s.trim().chars();
    (b'a'..=b'z')
        .map(|c| c as char)
        .map(|c| full_reaction(chars.clone().filter(|a| a.to_ascii_lowercase() != c)))
        .min()
        .unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let some = level1(&input);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&input);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test]
    fn level1_examples() {
        let input = "dabAcCaCBAcCcaDA";
        assert_eq!(level1(input), 10);

        let input = "aA";
        assert_eq!(level1(input), 0);

        let input = "abBA";
        assert_eq!(level1(input), 0);

        let input = "abAB";
        assert_eq!(level1(input), 4);

        let input = "aabAAB";
        assert_eq!(level1(input), 6);
    }

    #[test]
    fn level2_examples() {
        let input = "dabAcCaCBAcCcaDA";
        assert_eq!(level2(input), 4);
    }

    #[test]
    fn level1_regression() {
        assert_eq!(level1(INPUT), 9386);
    }

    #[test]
    fn level2_regression() {
        assert_eq!(level2(INPUT), 4876);
    }
}
