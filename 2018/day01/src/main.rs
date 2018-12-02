use std::{
    collections::{HashMap, HashSet},
    io::{self, Read, Write},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Credits: https://www.reddit.com/r/adventofcode/comments/a20646/2018_day_1_solutions/eaukxu5/
#[allow(dead_code)]
fn part2_fancy(s: &str) -> Result<i32> {
    use std::iter;

    let vals = parse_input(s)?.into_iter().scan(0, |state, x| {
        *state += x;
        Some(*state)
    });
    let mut cum_sums = iter::once(0).chain(vals).collect::<Vec<i32>>();
    let mut freq_set = HashSet::new();
    for &x in &cum_sums {
        if !freq_set.insert(x) {
            return Ok(x);
        }
    }

    let shift = cum_sums.pop().unwrap();
    if shift == 0 {
        return Ok(0);
    }

    let mut groups: HashMap<i32, Vec<(usize, i32)>> = HashMap::new();
    for (i, freq) in cum_sums.into_iter().enumerate() {
        groups
            .entry(freq % shift.abs())
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

    min_freq.ok_or_else(|| "no minimal frequency found".into())
}

fn parse_input(s: &str) -> Result<Vec<i32>> {
    s.lines()
        .map(|s| s.parse::<i32>())
        .collect::<std::result::Result<Vec<i32>, _>>()
        .map_err(Into::into)
}

fn part1(s: &str) -> Result<i32> {
    let mut total = 0;
    for l in s.lines() {
        total += l.parse::<i32>()?;
    }
    Ok(total)
}

fn part2(s: &str) -> Result<i32> {
    let vals = parse_input(s)?;
    let mut freq = 0i32;
    let mut seen = HashSet::new();
    seen.insert(0);

    for x in vals.into_iter().cycle() {
        freq += x;
        if !seen.insert(freq) {
            break;
        }
    }

    Ok(freq)
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let total_freq = part1(&input)?;
    writeln!(io::stderr(), "level 1: {}", total_freq)?;

    let double_freq = part2(&input)?;
    writeln!(io::stderr(), "level 2: {}", double_freq)?;

    writeln!(io::stdout(), "{}", double_freq)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    fn check_first_twice(input: &str, expected: i32) {
        let result = part2(input).unwrap();
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
        assert_eq!(part1(INPUT).unwrap(), 547);
    }

    #[test]
    fn part2_regresssion() {
        assert_eq!(part2(INPUT).unwrap(), 76414);
    }
}
