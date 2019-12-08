use std::{
    convert::TryFrom,
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

#[derive(Clone, Copy)]
enum Signal {
    Value(i32),
    Halting,
}

#[derive(Clone)]
struct IntCode(Vec<i32>);

impl FromStr for IntCode {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.trim()
            .split(',')
            .map(str::parse::<i32>)
            .collect::<Result<Vec<i32>, Self::Err>>()
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
        loop {
            let mut instruction = intcode[ip];
            let opcode = (instruction % 100) as u8;
            instruction /= 100;
            let mut mode = [0u8; 3];
            for i in 0..3 {
                mode[i] = (instruction % 10) as u8;
                if !(0..=1).contains(&mode[i]) {
                    err!("Unkown mode encountered: {}", mode[i])?
                }
                instruction /= 10;
            }

            match opcode {
                1 | 2 | 7 | 8 => {
                    let mut args = [0; 2];
                    for i in 0..2 {
                        let val = intcode[ip + i + 1];
                        args[i] = match mode[i] {
                            0 => intcode[val as usize],
                            1 => val,
                            _ => unreachable!(),
                        };
                    }

                    let out = intcode[ip + 3] as usize;
                    intcode[out] = match opcode {
                        1 => args[0] + args[1],
                        2 => args[0] * args[1],
                        7 =>
                            if args[0] < args[1] {
                                1
                            } else {
                                0
                            },
                        8 =>
                            if args[0] == args[1] {
                                1
                            } else {
                                0
                            },
                        _ => unreachable!(),
                    };

                    ip += 4;
                },
                3 => {
                    let address = intcode[ip + 1] as usize;
                    let val = match input.recv().unwrap() {
                        Signal::Value(x) => x,
                        Signal::Halting => break,
                    };

                    intcode[address] = val;
                    ip += 2;
                },
                4 => {
                    let address = intcode[ip + 1] as usize;
                    let out = intcode[address];
                    let _ = output.send(Signal::Value(out));
                    log::debug!("Output {}", out);
                    ip += 2;
                },
                5 | 6 => {
                    let mut args = [0; 2];
                    for i in 0..2 {
                        let val = intcode[ip + i + 1];
                        args[i] = match mode[i] {
                            0 => intcode[val as usize],
                            1 => val,
                            _ => unreachable!(),
                        };
                    }

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
                },
                99 => break,
                _ => err!("Unknown opcode encountered: {}", opcode)?,
            }
        }

        let _ = output.send(Signal::Halting);
        return Ok(());
    }
}

fn level1(intcode: &IntCode) -> aoc::Result<u32> {
    let mut phase_settings = vec![0, 1, 2, 3, 4];
    let heap = permutohedron::Heap::new(&mut phase_settings);
    let mut thruster_signal = 0;

    for permutation in heap {
        let mut amplified_input = 0;
        for phase in permutation {
            let mut ic = intcode.clone();
            let (input, rx) = mpsc::channel();
            let (tx, output) = mpsc::channel();
            input.send(Signal::Value(phase)).unwrap();
            input.send(Signal::Value(amplified_input)).unwrap();
            ic.run(rx, tx)?;
            amplified_input = match output.recv().unwrap() {
                Signal::Value(x) => x,
                Signal::Halting => err!("Amplifier halted before giving output")?,
            };
        }

        let output = u32::try_from(amplified_input)?;
        thruster_signal = u32::max(thruster_signal, output);
    }

    Ok(thruster_signal)
}

fn level2(intcode: &IntCode) -> aoc::Result<u32> {
    let mut phase_settings = vec![5, 6, 7, 8, 9];
    let mut thruster_signal = 0;
    let heap = permutohedron::Heap::new(&mut phase_settings);
    for permutation in heap {
        let (init_tx, init_rx) = mpsc::channel();
        let mut tx = init_tx.clone();
        let mut rx = init_rx;
        for i in 0..=4 {
            let mut ic = intcode.clone();
            let (new_tx, new_rx) = mpsc::channel();
            let cloned_tx = new_tx.clone();
            rayon::spawn(move || ic.run(rx, cloned_tx).unwrap());
            tx.send(Signal::Value(permutation[i])).unwrap();
            tx = new_tx;
            rx = new_rx;
        }

        init_tx.send(Signal::Value(0)).unwrap();
        let mut amplified_input = 0;
        for sig in rx.iter() {
            let _ = init_tx.send(sig);
            match sig {
                Signal::Value(x) => amplified_input = x,
                Signal::Halting => break,
            }
        }

        let result = u32::try_from(amplified_input)?;
        thruster_signal = u32::max(result, thruster_signal);
    }

    log::info!("{}", rayon::current_num_threads());
    Ok(thruster_signal)
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
    fn sanity() -> aoc::Result<()> {
        let input = INPUT.parse()?;
        let result = level1(&input)?;
        assert_eq!(result, 18812);

        let result = level2(&input)?;
        assert_eq!(result, 25534964);
        Ok(())
    }
}
