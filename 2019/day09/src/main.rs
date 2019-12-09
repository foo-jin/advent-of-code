use intcode::{self, Signal};
use std::io::{self, Read, Write};

fn run_with_input(ic: intcode::VM, input_value: intcode::Value) -> aoc::Result<intcode::Value> {
    let (input, output) = ic.spawn();
    input.send(Signal::Value(input_value))?;
    let mut result = None;
    for o in output {
        match o {
            Signal::Value(x) => result = Some(x),
            Signal::Halting if result == None => {
                return aoc::err!("BOOST halted before giving output")
            }
            Signal::Halting => break,
        }
    }
    Ok(result.unwrap())
}

fn level1(program: &str) -> aoc::Result<intcode::Value> {
    let mut vm = intcode::VM::new();
    vm.read_program(program)?;
    run_with_input(vm, 1)
}

fn level2(program: &str) -> aoc::Result<intcode::Value> {
    let mut vm = intcode::VM::new();
    vm.read_program(program)?;
    run_with_input(vm, 2)
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let some = level1(&input)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&input)?;
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
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
        let input = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99";
        let result = level1(&input)?;
        assert_eq!(result, 99);

        let input = "1102,34915192,34915192,7,4,7,99,0";
        let result = level1(&input)?;
        assert_eq!(result.to_string().len(), 16);

        let input = "104,1125899906842624,99";
        let result = level1(&input)?;
        assert_eq!(result, 1125899906842624);
        Ok(())
    }

    #[test_log::new]
    fn sanity() -> aoc::Result<()> {
        let result = level1(INPUT)?;
        assert_eq!(result, 4288078517);

        let result = level2(INPUT)?;
        assert_eq!(result, 69256);

        Ok(())
    }
}
