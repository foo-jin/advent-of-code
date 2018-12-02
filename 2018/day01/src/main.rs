#![feature(euclidean_division)]
use std::collections::{HashMap, HashSet};

const INPUT: &str = include_str!("../input.txt");

#[allow(dead_code)]
fn part1(s: &str) -> i32 {
    s.lines().map(|s| s.parse::<i32>().unwrap()).sum()
}

fn part2(s: &str) -> i32 {
    let mut freq = 0i32;
    let mut seen = HashSet::new();
    seen.insert(0);

    for x in s.lines().map(|s| s.parse::<i32>().unwrap()).cycle() {
        freq += x;
        if !seen.insert(freq) {
            break;
        }
    }

    freq
}

/// Credits: https://www.reddit.com/r/adventofcode/comments/a20646/2018_day_1_solutions/eaukxu5/
#[allow(dead_code)]
fn part2_fancy(s: &str) -> i32 {
    use std::iter;

    let vals = s
        .lines()
        .map(|s| s.parse::<i32>().unwrap())
        .scan(0, |state, x| {
            *state += x;
            Some(*state)
        });
    let mut cum_sums = iter::once(0).chain(vals).collect::<Vec<i32>>();
    let mut freq_set = HashSet::new();
    for &x in &cum_sums {
        if !freq_set.insert(x) {
            return x;
        }
    }

    let shift = cum_sums.pop().unwrap();
    if shift == 0 {
        return 0;
    }

    let mut groups: HashMap<i32, Vec<(usize, i32)>> = HashMap::new();
    for (i, freq) in cum_sums.into_iter().enumerate() {
        groups
            .entry(freq.mod_euc(shift))
            .or_default()
            .push((i, freq));
    }

    let mut min_index = 0;
    let mut min_diff = None;
    let mut min_freq = None;

    for group in groups.values_mut().filter(|g| g.len() > 1) {
        group.sort_unstable_by_key(|(_i, freq)| -*freq);
        for w in group.windows(2) {
            let cur = w[0];
            let prev = w[1];

            let diff = Some(cur.1 - prev.1);
            let (index, freq) = if shift > 0 {
                (prev.0, Some(cur.1))
            } else {
                (cur.0, Some(prev.1))
            };

            if min_diff.is_none() || diff < min_diff || (diff == min_diff && index < min_index) {
                min_index = index;
                min_diff = diff;
                min_freq = freq;
            }
        }
    }

    min_freq.unwrap()
}

fn main() -> Result<(), failure::Error> {
    // let freq = part1(INPUT);
    let freq = part2(INPUT);
    print!("{}", freq);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_first_twice(input: &str, expected: i32) {
        let result = part2(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn first_twice() {
        let input = "+1\n-1";
        check_first_twice(input, 0);

        let input = "+3\n+3\n+4\n-2\n-4";
        check_first_twice(input, 10);

        let input = "-6\n+3\n+8\n+5\n-6";
        check_first_twice(input, 5);

        let input = "+7\n+7\n-2\n-7\n-4";
        check_first_twice(input, 14);

        let input = "+1\n+1\n+10\n-9";
        check_first_twice(input, 12);
    }

    #[test]
    fn part1_regresssion() {
        assert_eq!(part1(INPUT), 547);
    }

    #[test]
    fn part2_regresssion() {
        assert_eq!(part2(INPUT), 76414);
    }
}
