use std::{str::FromStr, sync::mpsc};

const CODE_LEN: usize = 10_000;

pub type Value = i64;

#[derive(Clone, Copy)]
pub enum Signal {
    Value(Value),
    Halting,
}

#[derive(Default)]
pub struct VM {
    ip: usize,
    rp: Value,
    input: Option<mpsc::Receiver<Signal>>,
    output: Option<mpsc::Sender<Signal>>,
    mem: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        VM::default()
    }

    pub fn read_program(&mut self, program: &str) -> aoc::Result<()> {
        self.ip = 0;
        self.rp = 0;
        self.mem.clear();
        for num in program.trim().split(',') {
            let x = num.parse()?;
            self.mem.push(x);
        }
        self.mem.resize(CODE_LEN, Default::default());
        Ok(())
    }

    pub fn connect_io(
        &mut self,
        input: mpsc::Receiver<Signal>,
        output: mpsc::Sender<Signal>,
    ) -> aoc::Result<()> {
        self.input = Some(input);
        self.output = Some(output);
        Ok(())
    }

    pub fn setup_io(&mut self) -> (mpsc::Sender<Signal>, mpsc::Receiver<Signal>) {
        let (tx, input) = mpsc::channel();
        let (output, rx) = mpsc::channel();
        self.connect_io(input, output).expect("Failed to connect IO channels");
        (tx, rx)
    }

    pub fn run(&mut self) -> aoc::Result<()> {
        let input =
            self.input.as_ref().ok_or_else(|| aoc::format_err!("No input channel connected"))?;
        let output =
            self.output.as_ref().ok_or_else(|| aoc::format_err!("No output channel connected"))?;
        loop {
            let mut instruction = self.mem[self.ip];
            log::debug!("ip := {}\tinstr := {:05}\trelbase := {}", self.ip, instruction, self.rp);
            let opcode = (instruction % 100) as u8;
            instruction /= 100;
            let mut mode = [0u8; 3];
            let mut args = [0; 3];
            for i in 0..3 {
                mode[i] = (instruction % 10) as u8;
                let val = self.mem[self.ip + i + 1];
                let val_addr = val as usize;
                let rel_addr = (self.rp + val) as usize;
                args[i] = match mode[i] {
                    0 if val_addr < CODE_LEN => self.mem[val as usize],
                    2 if rel_addr < CODE_LEN => self.mem[(self.rp + val) as usize],
                    0 | 1 | 2 => val,
                    m => return aoc::err!("Unkown mode encountered: {}", m),
                };
                instruction /= 10;
            }

            match opcode {
                1 | 2 | 7 | 8 => {
                    let val = self.mem[self.ip + 3];
                    let address = match mode[2] {
                        0 => val,
                        1 => {
                            return aoc::err!(
                                "Writing results in immediate mode does not make sense."
                            )
                        }
                        2 => self.rp + val,
                        _ => unreachable!(),
                    } as usize;

                    self.mem[address] = match opcode {
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

                    log::debug!("intcode[{}] := {} = {}", address, calc, self.mem[address]);

                    self.ip += 4;
                }
                3 => {
                    let val = self.mem[self.ip + 1];
                    let address = match mode[0] {
                        0 => val,
                        1 => {
                            return aoc::err!(
                                "Reading input in immediate mode does not make sense."
                            )
                        }
                        2 => self.rp + val,
                        _ => unreachable!(),
                    };

                    let value = match input.recv().unwrap() {
                        Signal::Value(x) => x,
                        Signal::Halting => break,
                    };

                    self.mem[address as usize] = value;
                    log::debug!("intcode[{}] <- {}", address, value);
                    self.ip += 2;
                }
                4 => {
                    let out = args[0];
                    let _ = output.send(Signal::Value(out));
                    log::debug!("Output {}", out);
                    self.ip += 2;
                }
                5 | 6 => {
                    let b = match opcode {
                        5 => args[0] != 0,
                        6 => args[0] == 0,
                        _ => unreachable!(),
                    };

                    if b {
                        self.ip = args[1] as usize
                    } else {
                        self.ip += 3;
                    }
                }
                9 => {
                    let offset = args[0];
                    self.rp += offset;
                    log::debug!("relative_base += {}", offset);
                    self.ip += 2;
                }
                99 => break,
                _ => return aoc::err!("Unknown opcode encountered: {}", opcode),
            }
        }

        let _ = output.send(Signal::Halting);
        Ok(())
    }

    pub fn spawn(mut self) -> (mpsc::Sender<Signal>, mpsc::Receiver<Signal>) {
        let ends = self.setup_io();
        rayon::spawn(move || self.run().unwrap());
        ends
    }
}
