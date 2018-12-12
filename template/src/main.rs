use std::{
    error::Error,
    io::{self, Read, Write},
};

fn level1(s: &str) -> () {
    unimplemented!()
}

fn main() -> Result<(), Box<dyn Error>> {
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

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test]
    fn level1_examples() {
        assert_eq!(1, 1)
    }
}
