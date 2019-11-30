use std::{
    io::{self, Read, Write},
};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<std::error::Error>::from(format!($($tt)*))) }
}

macro_rules! format_err {
    ($($tt:tt)*) => { Box::<std::error::Error>::from(format!($($tt)*)) }
}

mod aoc {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let some = level1(&input);
    writeln!(io::stderr(), "level 1: {}", some)?;

    // let thing = level2(&input);
    // writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", some)?;
    Ok(())
}

fn level1(s: &str) -> () {
    unimplemented!()
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

    #[test_log::new]
    fn level1_examples() {
        assert_eq!(1, 1)
    }
}
