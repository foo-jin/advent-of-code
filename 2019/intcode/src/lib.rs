use std::{convert::TryFrom, sync::mpsc};

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
    instr: Instruction,
    input: Option<mpsc::Receiver<Signal>>,
    output: Option<mpsc::Sender<Signal>>,
    mem: Vec<Value>,
}

#[derive(Clone, Copy)]
enum Opcode {
    Add,
    Mul,
    Read,
    Write,
    Jit,
    Jif,
    Lt,
    Eq,
    Set,
    Halt,
}

#[derive(Clone, Copy)]
enum Mode {
    Positional,
    Immediate,
    Relative,
}

#[derive(Default)]
struct Instruction {
    mp: usize,
    modes: [Mode; 3],
}

impl TryFrom<Value> for Opcode {
    type Error = Box<dyn std::error::Error>;

    fn try_from(x: Value) -> Result<Self, Self::Error> {
        use Opcode::*;
        let opcode = match x {
            1 => Add,
            2 => Mul,
            3 => Read,
            4 => Write,
            5 => Jit,
            6 => Jif,
            7 => Lt,
            8 => Eq,
            9 => Set,
            99 => Halt,
            m => return aoc::err!("Unkown opcode encountered: {}", m),
        };

        Ok(opcode)
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Positional
    }
}

impl TryFrom<Value> for Mode {
    type Error = Box<dyn std::error::Error>;

    fn try_from(x: Value) -> Result<Self, Self::Error> {
        use Mode::*;
        let mode = match x {
            0 => Positional,
            1 => Immediate,
            2 => Relative,
            m => return aoc::err!("Unkown mode encountered: {}", m),
        };

        Ok(mode)
    }
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
        use Opcode::*;
        let input =
            self.input.take().ok_or_else(|| aoc::format_err!("No input channel connected"))?;
        let output =
            self.output.take().ok_or_else(|| aoc::format_err!("No output channel connected"))?;

        macro_rules! store {
            ($x:expr) => {
                let value = $x;
                let address = self.get_address()?;
                self.assign_expand(address, value);
            };
        }

        loop {
            log::debug!(
                "ip := {}\tinstr := {:05}\trelbase := {}",
                self.ip,
                self.mem[self.ip],
                self.rp
            );
            let opcode = self.get_opcode()?;
            match opcode {
                // 1
                Add => {
                    let result = self.get_value()?.checked_add(self.get_value()?).unwrap();
                    store!(result);
                }
                // 2
                Mul => {
                    let result = self.get_value()?.checked_mul(self.get_value()?).unwrap();
                    store!(result);
                }
                // 3
                Read => {
                    let value = match input.recv().unwrap() {
                        Signal::Value(x) => x,
                        Signal::Halting => break,
                    };
                    store!(value);
                }
                // 4
                Write => {
                    let _ = output.send(Signal::Value(self.get_value()?));
                }
                // 5, 6
                Jit | Jif => {
                    let val = self.get_value()?;
                    let cond = match opcode {
                        Jit => val != 0,
                        Jif => val == 0,
                        _ => unreachable!(),
                    };
                    let value = self.get_value()?;
                    if cond {
                        self.ip = usize::try_from(value)?;
                    }
                }
                // 7, 8
                Lt | Eq => {
                    let a = self.get_value()?;
                    let b = self.get_value()?;
                    let cond = match opcode {
                        Lt => a < b,
                        Eq => a == b,
                        _ => unreachable!(),
                    };
                    let address = self.get_address()?;
                    if cond {
                        self.assign_expand(address, 1);
                    } else {
                        self.assign_expand(address, 0);
                    }
                }
                // 9
                Set => self.rp += self.get_value()?,
                // 99
                Halt => break,
            }
        }

        let _ = output.send(Signal::Halting);
        Ok(())
    }

    fn get_opcode(&mut self) -> aoc::Result<Opcode> {
        let mut instruction = self.mem[self.ip];
        let opcode = Opcode::try_from(instruction % 100)?;
        let mut mode = [Mode::Positional; 3];
        instruction /= 100;
        for m in &mut mode {
            *m = Mode::try_from(instruction % 10)?;
            instruction /= 10;
        }

        self.instr = Instruction { mp: 0, modes: mode };
        self.ip += 1;
        Ok(opcode)
    }

    fn get_value(&mut self) -> aoc::Result<Value> {
        use Mode::*;
        let address = match self.instr.modes[self.instr.mp] {
            Positional | Relative => self.get_address()?,
            Immediate => {
                let ip = self.ip;
                self.ip += 1;
                self.instr.mp += 1;
                ip
            }
        };
        let value = self.mem.get(address).copied().unwrap_or_default();
        log::debug!("[{}] = {}", address, value);
        Ok(value)
    }

    fn get_address(&mut self) -> aoc::Result<usize> {
        use Mode::*;
        let address = match self.instr.modes[self.instr.mp] {
            Positional => usize::try_from(self.mem[self.ip])?,
            Relative => usize::try_from(self.rp + self.mem[self.ip])?,
            Immediate => {
                return aoc::err!("Writing results in immediate mode does not make sense.")
            }
        };
        self.ip += 1;
        self.instr.mp += 1;
        Ok(address)
    }

    fn assign_expand(&mut self, address: usize, value: Value) {
        if address >= self.mem.len() {
            self.mem.resize_with(address + 1, Default::default);
        }
        self.mem[address] = value;
        log::debug!("[{}] := {}", address, value);
    }

    pub fn spawn(mut self) -> (mpsc::Sender<Signal>, mpsc::Receiver<Signal>) {
        let ends = self.setup_io();
        rayon::spawn(move || self.run().unwrap());
        ends
    }
}
