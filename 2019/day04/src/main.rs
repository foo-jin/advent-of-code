use std::io::{self, Read, Write};

mod aoc {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}

fn solve() -> aoc::Result<()> {
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

fn level1(s: &str) -> usize {
    let ints = s
        .trim()
        .split('-')
        .map(str::parse::<u32>)
        .map(Result::unwrap)
        .collect::<Vec<u32>>();

    (ints[0]..=ints[1]).filter(|pw| is_password(*pw)).count()
}

fn is_password(mut pw: u32) -> bool {
    let mut adj = false;
    let mut decreasing = true;
    let mut prev = 10;
    while pw > 0 {
        let digit = pw % 10;
        decreasing &= digit <= prev;
        adj |= digit == prev;
        prev = digit;
        pw /= 10;
    }
    adj && decreasing
}

fn level2(s: &str) -> usize {
    let ints = s
        .trim()
        .split('-')
        .map(str::parse::<u32>)
        .map(Result::unwrap)
        .collect::<Vec<u32>>();

    (ints[0]..=ints[1]).filter(|pw| is_passwordv2(*pw)).count()
}

fn is_passwordv2(mut pw: u32) -> bool {
    let mut adjacent = false;
    let mut decreasing = true;
    let mut prev = 10;
    let mut grouplen = 0;
    while pw > 0 {
        let digit = pw % 10;
        decreasing &= digit <= prev;
        let adj = digit == prev;
        match (adj, grouplen) {
            (false, 1) => {
                adjacent = true;
                grouplen = 0
            }
            (false, _) => grouplen = 0,
            (true, _) => grouplen += 1,
        }
        prev = digit;
        pw /= 10;
    }
    if grouplen == 1 {
        adjacent = true
    }
    adjacent && decreasing
}

fn main() -> aoc::Result<()> {
    env_logger::init();
    if let Err(e) = solve() {
        let stderr = io::stderr();
        let mut w = stderr.lock();
        writeln!(w, "Error: {}", e)?;
        while let Some(e) = e.source() {
            writeln!(w, "\t{}", e)?;
        }

        std::process::exit(-1)
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test]
    fn level1_examples() {
        assert_eq!(is_password(111111), true);
        assert_eq!(is_password(223450), false);
        assert_eq!(is_password(123789), false);
    }

    #[test]
    fn level2_examples() {
        assert_eq!(is_passwordv2(111111), false);
        assert_eq!(is_passwordv2(223450), false);
        assert_eq!(is_passwordv2(123789), false);
        assert_eq!(is_passwordv2(112233), true);
        assert_eq!(is_passwordv2(123444), false);
        assert_eq!(is_passwordv2(111122), true);
        assert_eq!(is_passwordv2(112222), true);
    }
}
