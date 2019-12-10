use std::{convert::TryFrom, sync::mpsc};

pub type Value = i64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

    pub fn with_program(program: &str) -> aoc::Result<Self> {
        let mut vm = VM::new();
        vm.read_program(program)?;
        Ok(vm)
    }

    pub fn with_mem(mem: &[Value]) -> Self {
        let mut vm = VM::new();
        vm.read_mem(mem);
        vm
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

    pub fn read_mem(&mut self, mem: &[Value]) {
        self.ip = 0;
        self.rp = 0;
        self.mem.resize(mem.len(), 0);
        self.mem.copy_from_slice(mem);
    }

    pub fn mem(&self) -> &[Value] {
        &self.mem
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

    pub fn setup_io(
        &mut self,
    ) -> (mpsc::Sender<Signal>, mpsc::Receiver<Signal>) {
        let (tx, input) = mpsc::channel();
        let (output, rx) = mpsc::channel();
        self.connect_io(input, output).expect("Failed to connect IO channels");
        (tx, rx)
    }

    pub fn run(&mut self) -> aoc::Result<()> {
        use Opcode::*;
        let input = self
            .input
            .take()
            .ok_or_else(|| aoc::format_err!("No input channel connected"))?;
        let output = self
            .output
            .take()
            .ok_or_else(|| aoc::format_err!("No output channel connected"))?;

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
                    let result = self
                        .get_value()?
                        .checked_add(self.get_value()?)
                        .unwrap();
                    store!(result);
                }
                // 2
                Mul => {
                    let result = self
                        .get_value()?
                        .checked_mul(self.get_value()?)
                        .unwrap();
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
                return aoc::err!(
                    "Writing results in immediate mode does not make sense."
                )
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

impl Signal {
    pub fn is_value(&self) -> bool {
        match *self {
            Signal::Value(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test_log::new]
    fn add_mul() -> aoc::Result<()> {
        fn run_test(
            vm: &mut VM,
            input: &[Value],
            expected: &[Value],
        ) -> aoc::Result<()> {
            vm.read_mem(input);
            let _ = vm.setup_io();
            vm.run()?;
            assert_eq!(vm.mem.as_slice(), expected);
            Ok(())
        }

        let mut vm = VM::new();
        run_test(
            &mut vm,
            &[1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50],
            &[3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50],
        )?;
        run_test(&mut vm, &[1, 0, 0, 0, 99], &[2, 0, 0, 0, 99])?;
        run_test(&mut vm, &[2, 3, 0, 3, 99], &[2, 3, 0, 6, 99])?;
        run_test(&mut vm, &[2, 4, 4, 5, 99, 0], &[2, 4, 4, 5, 99, 9801])?;
        run_test(
            &mut vm,
            &[1, 1, 1, 4, 99, 5, 6, 0, 99],
            &[30, 1, 1, 4, 2, 5, 6, 0, 99],
        )?;
        Ok(())
    }

    #[test_log::new]
    fn io_jump() -> aoc::Result<()> {
        fn run_test(
            vm: &mut VM,
            program: &str,
            input_value: Value,
            expected: Value,
        ) -> aoc::Result<()> {
            vm.read_program(program)?;
            let (input, output) = vm.setup_io();
            input.send(Signal::Value(input_value))?;
            vm.run()?;
            let results =
                output.iter().filter(Signal::is_value).collect::<Vec<Signal>>();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0], Signal::Value(expected));
            Ok(())
        }

        let mut vm = VM::new();
        run_test(&mut vm, "3,9,8,9,10,9,4,9,99,-1,8", 8, 1)?;
        run_test(&mut vm, "3,9,8,9,10,9,4,9,99,-1,8", 7, 0)?;
        run_test(&mut vm, "3,9,7,9,10,9,4,9,99,-1,8", 7, 1)?;
        run_test(&mut vm, "3,9,7,9,10,9,4,9,99,-1,8", 8, 0)?;
        run_test(&mut vm, "3,9,7,9,10,9,4,9,99,-1,8", 8, 0)?;
        Ok(())
    }

    #[test_log::new]
    fn diagnostic_program() -> aoc::Result<()> {
        const DIAGNOSTIC: &str = include_str!("../../day05/input.txt");

        let mut vm = VM::with_program(DIAGNOSTIC)?;
        let (input, output) = vm.setup_io();
        input.send(Signal::Value(1))?;
        vm.run()?;
        let results =
            output.iter().filter(Signal::is_value).collect::<Vec<Signal>>();
        let n = results.len() - 1;
        for &test_result in &results[..n] {
            assert_eq!(test_result, Signal::Value(0));
        }
        assert_eq!(results[n], Signal::Value(4887191));

        vm.read_program(DIAGNOSTIC)?;
        let (input, output) = vm.setup_io();
        input.send(Signal::Value(5))?;
        vm.run()?;
        let results = output.iter().collect::<Vec<Signal>>();
        assert_eq!(results[0], Signal::Value(3419022));
        Ok(())
    }
}
