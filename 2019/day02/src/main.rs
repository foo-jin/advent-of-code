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
    let mut intcode = s
        .trim()
        .split(',')
        .map(str::parse::<usize>)
        .map(Result::unwrap)
        .collect::<Vec<usize>>();
    intcode[1] = 12;
    intcode[2] = 2;
    run_intcode(intcode)[0]
}

fn level2(s: &str) -> usize {
    let intcode = s
        .trim()
        .split(',')
        .map(str::parse::<usize>)
        .map(Result::unwrap)
        .collect::<Vec<usize>>();

    for noun in 0..99 {
        for verb in 0..99 {
            let mut current = intcode.clone();
            current[1] = noun;
            current[2] = verb;
            let result = run_intcode(current);
            if result[0] == 19690720 {
                return 100 * noun + verb;
            }
        }
    }
    panic!("No noun-verb combo found that produces the right value");
}

fn run_intcode(mut intcode: Vec<usize>) -> Vec<usize> {
    let mut head = 0;
    loop {
        match intcode[head] {
            1 | 2 => (),
            99 => return intcode,
            _ => panic!("Unknown opcode encountered: {}", intcode[head]),
        }

        let a = intcode[intcode[head + 1]];
        let b = intcode[intcode[head + 2]];
        let c = intcode[head + 3];

        intcode[c] = match intcode[head] {
            1 => a + b,
            2 => a * b,
            _ => unreachable!(),
        };

        head += 4;
    }
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
        let input = vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];
        assert_eq!(
            run_intcode(input),
            vec![3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50]
        );

        let input = vec![1, 0, 0, 0, 99];
        assert_eq!(run_intcode(input), vec![2, 0, 0, 0, 99]);

        let input = vec![2, 3, 0, 3, 99];
        assert_eq!(run_intcode(input), vec![2, 3, 0, 6, 99]);

        let input = vec![2, 4, 4, 5, 99, 0];
        assert_eq!(run_intcode(input), vec![2, 4, 4, 5, 99, 9801]);

        let input = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];
        assert_eq!(run_intcode(input), vec![30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }
}
