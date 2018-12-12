use std::{
    collections::HashMap,
    error::Error,
    io::{self, Read, Write},
};

fn parse_init(s: &str) -> Vec<bool> {
    s.trim_start_matches("initial state:")
        .trim()
        .chars()
        .map(|c| c == '#')
        .collect()
}

fn parse_rules(s: &str) -> HashMap<Vec<bool>, bool> {
    s.lines()
        .map(|l| {
            let mut parts = l.split("=>");
            let rule = parts
                .next()
                .unwrap()
                .trim()
                .chars()
                .map(|c| c == '#')
                .collect();
            let result = parts.next().unwrap().trim() == "#";
            (rule, result)
        })
        .collect()
}

fn parse_input(s: &str) -> (Vec<bool>, HashMap<Vec<bool>, bool>) {
    let mut parts = s.split("\n\n");
    let init = parts.next().map(parse_init).unwrap();
    let rules = parts.next().map(parse_rules).unwrap();
    (init, rules)
}

const PATTERN: u8 = 10;
fn simulate(init: &[bool], rules: &HashMap<Vec<bool>, bool>, gen: u64) -> i128 {
    let mut state = vec![false; 3];
    state.extend_from_slice(init);
    state.extend_from_slice(&[false; 3]);

    let mut diff = 0;
    let mut last = 0;
    let mut counter = 0;
    for i in 1..=gen {
        let mut next = vec![false; 3];
        let updated = state
            .windows(5)
            .map(|w| rules.get(w).map(|b| *b).unwrap_or(false));
        next.extend(updated);
        next.extend_from_slice(&[false; 3]);
        state = next;

        let offset = 3 + gen as i128;
        let score = state
            .iter()
            .enumerate()
            .filter(|(_i, b)| **b)
            .map(|(i, _b)| i as i128 - offset)
            .sum::<i128>();

        let ds = score - last;
        if ds == diff {
            counter += 1;
        } else {
            counter = 0;
        }

        if counter > PATTERN {
            let todo = (gen - i) as i128;
            return score + (todo * diff);
        }

        diff = ds;
        last = score;
    }

    last
}

fn level1(init: &[bool], rules: &HashMap<Vec<bool>, bool>) -> i128 {
    simulate(init, rules, 20)
}

fn level2(init: &[bool], rules: &HashMap<Vec<bool>, bool>) -> i128 {
    simulate(init, rules, 50_000_000_000)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let (init, rules) = parse_input(&input);

    let some = level1(&init, &rules);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&init, &rules);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");
    const EXAMPLE: &str = "initial state: #..#.#..##......###...###

...## => #
..#.. => #
.#... => #
.#.#. => #
.#.## => #
.##.. => #
.#### => #
#.#.# => #
#.### => #
##.#. => #
##.## => #
###.. => #
###.# => #
####. => #";

    #[test]
    fn level1_examples() {
        let (init, rules) = parse_input(EXAMPLE);
        assert_eq!(level1(&init, &rules), 325)
    }

    #[test]
    fn level1_regression() {
        let (init, rules) = parse_input(INPUT);
        assert_eq!(level1(&init, &rules), 1991)
    }

    #[test]
    fn level2_regression() {
        let (init, rules) = parse_input(INPUT);
        assert_eq!(level2(&init, &rules), 1_100_000_000_511)
    }
}
