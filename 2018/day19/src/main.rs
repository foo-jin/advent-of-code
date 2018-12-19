use std::{
    error::Error,
    io::{self, Read, Write},
    str::FromStr,
};

macro_rules! err {
    ($($tt:tt)*) => {
        Err(format_err!($($tt)*))
    }
}

macro_rules! format_err {
    ($($tt:tt)*) => { Box::<Error>::from(format!($($tt)*)) }
}

fn get_next<T>(lines: &mut impl Iterator<Item = T>) -> Result<T, Box<dyn Error>> {
    lines
        .next()
        .ok_or_else(|| format_err!("unexpected end of input"))
}

type Reg = usize;
type Val = usize;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum OpCode {
    Addr(Reg, Reg, Reg),
    Addi(Reg, Val, Reg),
    Mulr(Reg, Reg, Reg),
    Muli(Reg, Val, Reg),
    Banr(Reg, Reg, Reg),
    Bani(Reg, Val, Reg),
    Borr(Reg, Reg, Reg),
    Bori(Reg, Val, Reg),
    Setr(Reg, Val, Reg),
    Seti(Val, Val, Reg),
    Gtir(Val, Reg, Reg),
    Gtri(Reg, Val, Reg),
    Gtrr(Reg, Reg, Reg),
    Eqir(Val, Reg, Reg),
    Eqri(Reg, Val, Reg),
    Eqrr(Reg, Reg, Reg),
}

impl OpCode {
    fn get_constructor(ident: &str) -> Result<impl Fn(Val, Val, Val) -> OpCode, Box<dyn Error>> {
        use self::OpCode::*;

        let f = match ident {
            "addr" => Addr,
            "addi" => Addi,
            "mulr" => Mulr,
            "muli" => Muli,
            "banr" => Banr,
            "bani" => Bani,
            "borr" => Borr,
            "bori" => Bori,
            "setr" => Setr,
            "seti" => Seti,
            "gtir" => Gtir,
            "gtri" => Gtri,
            "gtrr" => Gtrr,
            "eqir" => Eqir,
            "eqri" => Eqri,
            "eqrr" => Eqrr,
            _ => return err!("unkown ident"),
        };

        Ok(f)
    }
}

fn parse_opcodes(s: &str) -> Result<Vec<OpCode>, Box<dyn Error>> {
    fn parse_instruction(l: &str) -> Result<OpCode, Box<dyn Error>> {
        let mut parts = l.split_whitespace();
        let ident = get_next(&mut parts)?;

        let mut parts = parts.map(Val::from_str);
        let a = get_next(&mut parts)??;
        let b = get_next(&mut parts)??;
        let c = get_next(&mut parts)??;
        let instruction = OpCode::get_constructor(ident)?(a, b, c);
        Ok(instruction)
    }

    let mut instructions = Vec::new();
    for l in s.lines() {
        let instr = parse_instruction(l)?;
        instructions.push(instr);
    }

    Ok(instructions)
}

impl FromStr for Program {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(r"([-\d]+)")?;

        let mut parts = s.trim().splitn(2, "\n");
        let first = get_next(&mut parts)?;
        let caps = re
            .captures(first)
            .ok_or_else(|| format_err!("expected a number on line 1"))?;
        let ip_reg = caps[0].parse()?;

        let program = get_next(&mut parts)?;
        let instr = parse_opcodes(program)?.into_boxed_slice();

        let result = Program {
            ip_reg,
            instr,
            ..Program::default()
        };

        Ok(result)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Program {
    ip: usize,
    ip_reg: Reg,
    regs: [Val; 6],
    instr: Box<[OpCode]>,
}

impl Program {
    fn step(&mut self) {
        use self::OpCode::*;

        let regs = &mut self.regs;
        regs[self.ip_reg] = self.ip as Val;
        match self.instr[self.ip] {
            Addr(a, b, c) => regs[c] = regs[a] + regs[b],
            Addi(a, b, c) => regs[c] = regs[a] + b,
            Mulr(a, b, c) => regs[c] = regs[a] * regs[b],
            Muli(a, b, c) => regs[c] = regs[a] * b,
            Banr(a, b, c) => regs[c] = regs[a] & regs[b],
            Bani(a, b, c) => regs[c] = regs[a] & b,
            Borr(a, b, c) => regs[c] = regs[a] | regs[b],
            Bori(a, b, c) => regs[c] = regs[a] | b,
            Setr(a, _, c) => regs[c] = regs[a],
            Seti(a, _, c) => regs[c] = a,
            Gtir(a, b, c) => regs[c] = if a > regs[b] { 1 } else { 0 },
            Gtri(a, b, c) => regs[c] = if regs[a] > b { 1 } else { 0 },
            Gtrr(a, b, c) => regs[c] = if regs[a] > regs[b] { 1 } else { 0 },
            Eqir(a, b, c) => regs[c] = if a == regs[b] { 1 } else { 0 },
            Eqri(a, b, c) => regs[c] = if regs[a] == b { 1 } else { 0 },
            Eqrr(a, b, c) => regs[c] = if regs[a] == regs[b] { 1 } else { 0 },
        }

        self.ip = regs[self.ip_reg] as usize;
        self.ip += 1;
    }

    fn execute_fully(&mut self) {
        while self.ip < self.instr.len() {
            self.step();
        }
    }
}

fn level1(mut program: Program) -> Val {
    program.execute_fully();
    program.regs[0]
}

fn level2(mut program: Program) -> Val {
    program.regs[0] = 1;
    while program.regs[program.ip_reg] != 1 {
        program.step();
    }
    let seed = *program.regs.iter().max().unwrap();
    (1..=seed).filter(|k| seed % k == 0).sum()
}

fn solve() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let program = input.parse::<Program>()?;

    let some = level1(program.clone());
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(program);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
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
    const EX: &str = "
#ip 0
seti 5 0 1
seti 6 0 2
addi 0 1 0
addr 1 2 3
setr 1 0 0
seti 8 0 4
seti 9 0 5
";

    #[test]
    fn level1_examples() {
        let program = EX.parse().unwrap();
        assert_eq!(level1(program), 6);
    }

    #[test]
    fn level1_regression() {
        let program = INPUT.parse().unwrap();
        assert_eq!(level1(program), 2160);
    }

    #[test]
    fn level2_regression() {
        let program = INPUT.parse().unwrap();
        assert_eq!(level2(program), 25945920);
    }
}
