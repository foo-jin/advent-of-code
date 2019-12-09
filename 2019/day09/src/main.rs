use std::{
    io::{self, Read, Write},
    str::FromStr,
    sync::mpsc,
};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn std::error::Error>::from(format!($($tt)*))) }
}

mod aoc {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}

const CODE_LEN: usize = 10_000;

type Val = i64;

#[derive(Clone, Copy)]
enum Signal {
    Value(Val),
    Halting,
}

#[derive(Clone)]
struct IntCode(Vec<Val>);

impl FromStr for IntCode {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.trim()
            .split(',')
            .map(str::parse::<Val>)
            .collect::<Result<Vec<Val>, Self::Err>>()
            .map(|mut xs| {
                if xs.len() < CODE_LEN {
                    xs.resize_with(CODE_LEN, Default::default);
                }
                xs
            })
            .map(IntCode)
    }
}

impl IntCode {
    fn run(
        &mut self,
        input: mpsc::Receiver<Signal>,
        output: mpsc::Sender<Signal>,
    ) -> aoc::Result<()> {
        let intcode = &mut self.0;
        let mut ip = 0;
        let mut relative_base = 0;
        loop {
            let mut instruction = intcode[ip];
            log::debug!("ip := {}\tinstr := {:05}\trelbase := {}", ip, instruction, relative_base);
            let opcode = (instruction % 100) as u8;
            instruction /= 100;
            let mut mode = [0u8; 3];
            let mut args = [0; 3];
            for i in 0..3 {
                mode[i] = (instruction % 10) as u8;
                let val = intcode[ip + i + 1];
                let val_addr = val as usize;
                let rel_addr = (relative_base + val) as usize;
                args[i] = match mode[i] {
                    0 if val_addr < CODE_LEN => intcode[val as usize],
                    2 if rel_addr < CODE_LEN => intcode[(relative_base + val) as usize],
                    0 | 1 | 2 => val,
                    m => return err!("Unkown mode encountered: {}", m),
                };
                instruction /= 10;
            }

            match opcode {
                1 | 2 | 7 | 8 => {
                    let val = intcode[ip + 3];
                    let address = match mode[2] {
                        0 => val,
                        1 => return err!("Writing results in immediate mode does not make sense."),
                        2 => relative_base + val,
                        _ => unreachable!(),
                    } as usize;

                    intcode[address] = match opcode {
                        1 => args[0].checked_add(args[1]).unwrap(),
                        2 => args[0].checked_mul(args[1]).unwrap(),
                        7 => {
                            if args[0] < args[1] {
                                1
                            } else {
                                0
                            }
                        }
                        8 => {
                            if args[0] == args[1] {
                                1
                            } else {
                                0
                            }
                        }
                        _ => unreachable!(),
                    };

                    let calc = match opcode {
                        1 => format!("{} + {}", args[0], args[1]),
                        2 => format!("{} * {}", args[0], args[1]),
                        7 => format!("{} < {}", args[0], args[1]),
                        8 => format!("{} == {}", args[0], args[1]),
                        _ => unreachable!(),
                    };

                    log::debug!("intcode[{}] := {} = {}", address, calc, intcode[address]);

                    ip += 4;
                }
                3 => {
                    let val = intcode[ip + 1];
                    let address = match mode[0] {
                        0 => val,
                        1 => return err!("Reading input in immediate mode does not make sense."),
                        2 => relative_base + val,
                        _ => unreachable!(),
                    };

                    let value = match input.recv().unwrap() {
                        Signal::Value(x) => x,
                        Signal::Halting => break,
                    };

                    intcode[address as usize] = value;
                    log::debug!("intcode[{}] <- {}", address, value);
                    ip += 2;
                }
                4 => {
                    let out = args[0];
                    let _ = output.send(Signal::Value(out));
                    log::debug!("Output {}", out);
                    ip += 2;
                }
                5 | 6 => {
                    let b = match opcode {
                        5 => args[0] != 0,
                        6 => args[0] == 0,
                        _ => unreachable!(),
                    };

                    if b {
                        ip = args[1] as usize
                    } else {
                        ip += 3;
                    }
                }
                9 => {
                    let offset = args[0];
                    relative_base += offset;
                    log::debug!("relative_base += {}", offset);
                    ip += 2;
                }
                99 => break,
                _ => return err!("Unknown opcode encountered: {}", opcode),
            }
        }

        let _ = output.send(Signal::Halting);
        Ok(())
    }

    fn spawn(mut self) -> (mpsc::Sender<Signal>, mpsc::Receiver<Signal>) {
        let (input, rx) = mpsc::channel();
        let (tx, output) = mpsc::channel();
        rayon::spawn(move || self.run(rx, tx).unwrap());
        (input, output)
    }
}

fn run_with_input(ic: IntCode, input_value: Val) -> aoc::Result<Val> {
    let (input, output) = ic.spawn();
    input.send(Signal::Value(input_value))?;
    let mut result = None;
    for o in output {
        match o {
            Signal::Value(x) => result = Some(x),
            Signal::Halting if result == None => return err!("BOOST halted before giving output"),
            Signal::Halting => break,
        }
    }
    Ok(result.unwrap())
}

fn level1(intcode: &IntCode) -> aoc::Result<Val> {
    run_with_input(intcode.clone(), 1)
}

fn level2(intcode: &IntCode) -> aoc::Result<Val> {
    run_with_input(intcode.clone(), 2)
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let parsed = input.parse()?;

    let some = level1(&parsed)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&parsed)?;
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
        let input = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99".parse()?;
        let result = level1(&input)?;
        assert_eq!(result, 99);

        let input = "1102,34915192,34915192,7,4,7,99,0".parse()?;
        let result = level1(&input)?;
        assert_eq!(result.to_string().len(), 16);

        let input = "104,1125899906842624,99".parse()?;
        let result = level1(&input)?;
        assert_eq!(result, 1125899906842624);
        Ok(())
    }

    #[test_log::new]
    fn sanity() -> aoc::Result<()> {
        let input = INPUT.parse()?;
        let result = level1(&input)?;
        assert_eq!(result, 4288078517);

        let result = level2(&input)?;
        assert_eq!(result, 69256);

        Ok(())
    }
}
