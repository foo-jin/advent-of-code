use std::collections::HashMap;

const INPUT: &str = include_str!("../input.txt");

#[allow(dead_code)]
fn part1(s: &str) -> u32 {
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

fn part2(s: &str) -> Option<String> {
    use std::iter;

    let mut lines = s.lines().collect::<Vec<&str>>();
    lines.sort_unstable();

    for (l1, l2) in lines.iter().flat_map(|l| iter::repeat(l).zip(lines.iter())) {
        let zipped = l1.chars().zip(l2.chars());
        if zipped.clone().filter(|(a, b)| a != b).count() == 1 {
            let substring = zipped.filter(|(a, b)| a == b).map(|(a, _b)| a).collect();
            return Some(substring);
        }
    }

    None
}

fn main() -> Result<(), failure::Error> {
    // let checksum = part1(INPUT);
    let commons = part2(INPUT).unwrap();
    eprintln!("result: {}", commons);
    print!("{}", commons);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1_example() {
        let input = "abcdef\nbababc\nabbcde\nabcccd\naabcdd\nabcdee\nababab";
        assert_eq!(part1(input), 12);
    }

    #[test]
    fn part2_example() {
        let input = "abcde\nfghij\nklmno\npqrst\nfguij\naxcye\nwvxyz";
        assert_eq!(part2(input), Some("fgij".to_string()));
    }

    #[test]
    fn part1_regression() {
        assert_eq!(part1(INPUT), 6150);
    }

    #[test]
    fn part2_regression() {
        assert_eq!(part2(INPUT), Some("rteotyxzbodglnpkudawhijsc".to_string()));
    }
}
