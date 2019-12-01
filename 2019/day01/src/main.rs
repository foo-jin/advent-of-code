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

fn level1(s: &str) -> u32 {
    s.trim()
        .split_whitespace()
        .map(str::parse::<u32>)
        .map(Result::unwrap)
        .map(|w| w / 3 - 2)
        .sum()
}

fn level2(s: &str) -> u32 {
    s.trim()
        .split_whitespace()
        .map(str::parse::<i32>)
        .map(Result::unwrap)
        .map(|w| {
            let mut mass: i32 = (w / 3) - 2;
            let mut diff = mass / 3 - 2;
            while diff > 0 {
                mass += diff;
                diff = i32::max(0, diff / 3 - 2);
            }
	    mass as u32
        })
        .sum()
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
    fn level2_examples() {
        assert_eq!(level2("14"), 2);
        assert_eq!(level2("1969"), 966);
        assert_eq!(level2("100756"), 50346);
    }

    #[test]
    fn level1_sanity() {
        assert_eq!(level1(INPUT), 3324332);
    }

    #[test]
    fn level2_sanity() {
        assert_eq!(level2(INPUT), 4983626);
    }
}
