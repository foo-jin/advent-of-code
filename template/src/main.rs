use std::{
    io::{self, Read, Write},
};

fn parse(s: String) -> aoc::Result<Input> {
    s
}

fn level1(input: ..) -> aoc::Result<()> {
    Ok(())
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let parsed = parse(input)?;

    let some = level1(&parsed)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    // let thing = level2(&parsed)?;
    // writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", some)?;
    Ok(())
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
    fn level1_examples() -> aoc::Result<()> {
	let input = parse("asdf")?;
	let result = level1(&input)?;
        assert_eq!(result, ());
	Ok(())
    }
}
