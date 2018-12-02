use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

fn level1(s: &str) -> u32 {
    let mut twos = 0;
    let mut threes = 0;
    for l in s.lines() {
        let mut counts = HashMap::with_capacity(26);
        for c in l.chars() {
            *counts.entry(c).or_insert(0u32) += 1;
        }

        if counts.values().any(|k| *k == 2) {
            twos += 1;
        }

        if counts.values().any(|k| *k == 3) {
            threes += 1;
        }
    }

    twos * threes
}

fn level2(s: &str) -> Option<String> {
    let mut lines = s.lines().collect::<Vec<&str>>();
    lines.sort_unstable();

    for (i, l1) in lines.iter().enumerate() {
        for l2 in lines.iter().skip(i) {
            let zipped = l1.chars().zip(l2.chars());
            if zipped.clone().filter(|(a, b)| a != b).count() == 1 {
                let substring = zipped.filter(|(a, b)| a == b).map(|(a, _b)| a).collect();
                return Some(substring);
            }
        }
    }

    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let checksum = level1(&input);
    writeln!(io::stderr(), "level 1: {}", checksum)?;

    let commons = level2(&input).ok_or_else(|| "failed to find the two correct box IDs")?;
    writeln!(io::stderr(), "level 2: {}", commons)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", commons)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test]
    fn level1_example() {
        let input = "abcdef\nbababc\nabbcde\nabcccd\naabcdd\nabcdee\nababab";
        assert_eq!(level1(input), 12);
    }

    #[test]
    fn level2_example() {
        let input = "abcde\nfghij\nklmno\npqrst\nfguij\naxcye\nwvxyz";
        assert_eq!(level2(input), Some("fgij".to_string()));
    }

    #[test]
    fn level1_regression() {
        assert_eq!(level1(INPUT), 6150);
    }

    #[test]
    fn level2_regression() {
        assert_eq!(level2(INPUT), Some("rteotyxzbodglnpkudawhijsc".to_string()));
    }
}
