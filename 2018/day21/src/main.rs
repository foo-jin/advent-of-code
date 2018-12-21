use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    io::{self, Read, Write},
    ops,
    str::FromStr,
};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<std::error::Error>::from(format!($($tt)*))) }
}

macro_rules! format_err {
    ($($tt:tt)*) => { Box::<std::error::Error>::from(format!($($tt)*)) }
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Value = u64;

fn solve() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let program = input.parse::<Program>()?;

    let some = level1(&program);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&program);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", some)?;
    Ok(())
}

fn level1(prog: &Program) -> Value {
    let mut vm = VM::default();
    vm.find_shortest(prog)
}

fn level2(prog: &Program) -> Value {
    let mut vm = VM::default();
    vm.find_longest(prog)
}

#[derive(Clone, Debug, Default)]
struct VM {
    registers: Registers,
    ip: usize,
    ipc: u64,
}

impl VM {
    fn step(&mut self, prog: &Program) -> bool {
        let ipreg = prog.ipreg;
        if let Some(instr) = prog.instr.get(self.ip) {
            self.ipc += 1;
            self.registers[ipreg] = self.ip as Value;
            instr.exec(&mut self.registers);
            self.ip = self.registers[ipreg] as usize + 1;

            false
        } else {
            true
        }
    }

    fn find_longest(&mut self, prog: &Program) -> Value {
        let mut conditions = HashMap::new();
        let mut best = 0;
        while !self.step(prog) {
            if self.ip == 28 {
                let val = self.registers[Register::R3];
                match conditions.entry(val) {
                    Entry::Vacant(e) => {
                        e.insert(val);
                        best = val;
                        log::trace!("NEW ipc: {}, {}", self.ipc, val);
                    }
                    Entry::Occupied(_) => {
                        log::trace!("DUPL ipc: {}, {}", self.ipc, val);
                        return best;
                    }
                }
            } else {
                log::trace!("ip: {}, {:?}", self.ip + 1, self.registers.0);
            }
        }
        unreachable!()
    }

    fn find_shortest(&mut self, prog: &Program) -> Value {
        while !self.step(prog) {
            if self.ip == 28 {
                return self.registers[Register::R3];
            }
        }
        unreachable!()
    }
}

struct Program {
    ipreg: Register,
    instr: Vec<Instruction>,
}

impl FromStr for Program {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self> {
        let mut program = Program {
            ipreg: Register::R0,
            instr: vec![],
        };
        for line in s.trim().lines() {
            if line.starts_with("#ip ") {
                let v = line[4..].parse::<Value>()?;
                program.ipreg = Register::from_value(v)?;
            } else {
                program.instr.push(line.parse()?);
            }
        }
        Ok(program)
    }
}

#[derive(Clone, Debug, Default)]
struct Registers([Value; 6]);

#[derive(Clone, Copy, Debug)]
enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
}

impl Register {
    fn as_index(&self) -> usize {
        match self {
            Register::R0 => 0,
            Register::R1 => 1,
            Register::R2 => 2,
            Register::R3 => 3,
            Register::R4 => 4,
            Register::R5 => 5,
        }
    }

    fn from_value(v: Value) -> Result<Self> {
        let r = match v {
            0 => Register::R0,
            1 => Register::R1,
            2 => Register::R2,
            3 => Register::R3,
            4 => Register::R4,
            5 => Register::R5,
            _ => return err!("invalid register number: {}", v),
        };

        Ok(r)
    }
}

impl ops::Index<Register> for Registers {
    type Output = Value;

    fn index(&self, r: Register) -> &Value {
        let i = r.as_index();
        self.0.index(i)
    }
}

impl ops::IndexMut<Register> for Registers {
    fn index_mut(&mut self, r: Register) -> &mut Value {
        let i = r.as_index();
        self.0.index_mut(i)
    }
}

#[derive(Clone, Copy, Debug)]
struct Instruction {
    output: Register,
    op: Opcode,
}

#[derive(Clone, Copy, Debug)]
enum Opcode {
    Addr { a: Register, b: Register },
    Addi { a: Register, b: Value },
    Mulr { a: Register, b: Register },
    Muli { a: Register, b: Value },
    Banr { a: Register, b: Register },
    Bani { a: Register, b: Value },
    Borr { a: Register, b: Register },
    Bori { a: Register, b: Value },
    Setr { a: Register },
    Seti { a: Value },
    Gtir { a: Value, b: Register },
    Gtri { a: Register, b: Value },
    Gtrr { a: Register, b: Register },
    Eqir { a: Value, b: Register },
    Eqri { a: Register, b: Value },
    Eqrr { a: Register, b: Register },
}

impl Instruction {
    fn exec(&self, regs: &mut Registers) {
        use self::Opcode::*;

        let result = match self.op {
            Addr { a, b } => regs[a] + regs[b],
            Addi { a, b } => regs[a] + b,
            Mulr { a, b } => regs[a] * regs[b],
            Muli { a, b } => regs[a] * b,
            Banr { a, b } => regs[a] & regs[b],
            Bani { a, b } => regs[a] & b,
            Borr { a, b } => regs[a] | regs[b],
            Bori { a, b } => regs[a] | b,
            Setr { a } => regs[a],
            Seti { a } => a,
            Gtir { a, b } => {
                if a > regs[b] {
                    1
                } else {
                    0
                }
            }
            Gtri { a, b } => {
                if regs[a] > b {
                    1
                } else {
                    0
                }
            }
            Gtrr { a, b } => {
                if regs[a] > regs[b] {
                    1
                } else {
                    0
                }
            }
            Eqir { a, b } => {
                if a == regs[b] {
                    1
                } else {
                    0
                }
            }
            Eqri { a, b } => {
                if regs[a] == b {
                    1
                } else {
                    0
                }
            }
            Eqrr { a, b } => {
                if regs[a] == regs[b] {
                    1
                } else {
                    0
                }
            }
        };

        regs[self.output] = result;
    }
}

impl FromStr for Instruction {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self> {
        use self::Opcode::*;
        use regex::Regex;

        lazy_static::lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?P<opcode>[a-z]+) (?P<a>[0-9]+) (?P<b>[0-9]+) (?P<c>[0-9]+)"
            ).unwrap();
        }

        let caps = RE
            .captures(s)
            .ok_or_else(|| format_err!("invalid instruction: '{:?}'", s))?;
        let (a, b, c) = (caps["a"].parse()?, caps["b"].parse()?, caps["c"].parse()?);
        let mkreg = Register::from_value;
        let opcode = match &caps["opcode"] {
            "addr" => Addr {
                a: mkreg(a)?,
                b: mkreg(b)?,
            },
            "addi" => Addi { a: mkreg(a)?, b },
            "mulr" => Mulr {
                a: mkreg(a)?,
                b: mkreg(b)?,
            },
            "muli" => Muli { a: mkreg(a)?, b },
            "banr" => Banr {
                a: mkreg(a)?,
                b: mkreg(b)?,
            },
            "bani" => Bani { a: mkreg(a)?, b },
            "borr" => Borr {
                a: mkreg(a)?,
                b: mkreg(b)?,
            },
            "bori" => Bori { a: mkreg(a)?, b },
            "setr" => Setr { a: mkreg(a)? },
            "seti" => Seti { a },
            "gtir" => Gtir { a, b: mkreg(b)? },
            "gtri" => Gtri { a: mkreg(a)?, b },
            "gtrr" => Gtrr {
                a: mkreg(a)?,
                b: mkreg(b)?,
            },
            "eqir" => Eqir { a, b: mkreg(b)? },
            "eqri" => Eqri { a: mkreg(a)?, b },
            "eqrr" => Eqrr {
                a: mkreg(a)?,
                b: mkreg(b)?,
            },
            unk => return err!("unknown opcode: {:?}", unk),
        };

        let instr = Instruction {
            output: Register::from_value(c)?,
            op: opcode,
        };
        Ok(instr)
    }
}

fn main() -> Result<()> {
    env_logger::init();
    if let Err(e) = solve() {
        let stderr = io::stderr();
        let mut w = stderr.lock();
        writeln!(w, "Error: {}", e)?;
        while let Some(e) = e.source() {
            writeln!(w, "\t{}", e)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test]
    fn level1_regression() {
        let prog = INPUT.parse().unwrap();
        assert_eq!(level1(&prog), 4797782)
    }

    #[test]
    fn level2_regression() {
        let prog = INPUT.parse().unwrap();
        assert_eq!(level2(&prog), 6086461)
    }
}
