use std::{
    collections::HashSet,
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

type Reg = usize;
type Val = u16;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Registers([Val; 4]);

impl Registers {
    fn new() -> Self {
        Self::default()
    }

    fn copy_execute(&self, instr: OpCode) -> Self {
        let mut copy = *self;
        copy.execute(instr);
        copy
    }

    fn execute(&mut self, instr: OpCode) {
        use self::OpCode::*;
        let regs = &mut self.0;
        match instr {
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
    }
}

impl FromStr for Registers {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut xs = s
            .trim_matches(|c: char| c.is_whitespace() || c == '[' || c == ']')
            .split(", ")
            .map(Val::from_str);

        let mut regs = [0; 4];
        for i in 0..regs.len() {
            regs[i] = xs
                .next()
                .ok_or_else(|| format_err!("unexpected end of input"))??;
        }

        Ok(Registers(regs))
    }
}

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
    fn to_ident(&self) -> &'static str {
        use self::OpCode::*;

        match self {
            Addr(_, _, _) => "addr",
            Addi(_, _, _) => "addi",
            Mulr(_, _, _) => "mulr",
            Muli(_, _, _) => "muli",
            Banr(_, _, _) => "banr",
            Bani(_, _, _) => "bani",
            Borr(_, _, _) => "borr",
            Bori(_, _, _) => "bori",
            Setr(_, _, _) => "setr",
            Seti(_, _, _) => "seti",
            Gtir(_, _, _) => "gtir",
            Gtri(_, _, _) => "gtri",
            Gtrr(_, _, _) => "gtrr",
            Eqir(_, _, _) => "eqir",
            Eqri(_, _, _) => "eqri",
            Eqrr(_, _, _) => "eqrr",
        }
    }

    fn get_constructor(
        ident: &str,
    ) -> Result<Box<dyn Fn(Val, Val, Val) -> OpCode>, Box<dyn Error>> {
        fn cast_args<T, A, B, C>(
            f: impl Fn(A, B, C) -> T + 'static,
        ) -> Box<dyn Fn(Val, Val, Val) -> T>
        where
            A: From<Val>,
            B: From<Val>,
            C: From<Val>,
        {
            Box::new(move |a, b, c| f(A::from(a), B::from(b), C::from(c)))
        }
        use self::OpCode::*;

        let f = match ident {
            "addr" => cast_args(Addr),
            "addi" => cast_args(Addi),
            "mulr" => cast_args(Mulr),
            "muli" => cast_args(Muli),
            "banr" => cast_args(Banr),
            "bani" => cast_args(Bani),
            "borr" => cast_args(Borr),
            "bori" => cast_args(Bori),
            "setr" => cast_args(Setr),
            "seti" => cast_args(Seti),
            "gtir" => cast_args(Gtir),
            "gtri" => cast_args(Gtri),
            "gtrr" => cast_args(Gtrr),
            "eqir" => cast_args(Eqir),
            "eqri" => cast_args(Eqri),
            "eqrr" => cast_args(Eqrr),
            _ => return err!("unkown ident"),
        };

        Ok(f)
    }
}

struct Sample {
    code: Val,
    before: Registers,
    after: Registers,
    a: Val,
    b: Val,
    c: Val,
}

impl Sample {
    fn gen_opcodes(&self) -> impl Iterator<Item = OpCode> {
        use self::OpCode::*;
        let a = self.a;
        let ra = a as usize;
        let b = self.b;
        let rb = b as usize;
        let c = self.c as usize;

        let items = vec![
            Addr(ra, rb, c),
            Addi(ra, b, c),
            Mulr(ra, rb, c),
            Muli(ra, b, c),
            Banr(ra, rb, c),
            Bani(ra, b, c),
            Borr(ra, rb, c),
            Bori(ra, b, c),
            Setr(ra, b, c),
            Seti(a, b, c),
            Gtir(a, rb, c),
            Gtri(ra, b, c),
            Gtrr(ra, rb, c),
            Eqir(a, rb, c),
            Eqri(ra, b, c),
            Eqrr(ra, rb, c),
        ];

        items.into_iter()
    }

    fn matching_opcodes(&self) -> impl Iterator<Item = OpCode> + '_ {
        self.gen_opcodes()
            .map(move |op| (op, self.before.copy_execute(op)))
            .filter(move |(_op, reg)| *reg == self.after)
            .map(|(op, _reg)| op)
    }
}

fn get_next<T>(
    lines: &mut impl Iterator<Item = T>,
) -> Result<T, Box<dyn Error>> {
    lines.next().ok_or_else(|| format_err!("unexpected end of input"))
}

impl FromStr for Sample {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_reg(line: &str) -> Result<Registers, Box<dyn Error>> {
            let mut parts = line.split(": ");
            let _ = parts.next();
            let s = get_next(&mut parts)?;
            s.parse::<Registers>()
        }

        let mut lines = s.lines();
        let before = {
            let l = get_next(&mut lines)?;
            parse_reg(l)?
        };

        let (code, a, b, c) = {
            let l = get_next(&mut lines)?;
            let mut parts = l.split_whitespace().map(Val::from_str);
            let code = get_next(&mut parts)??;
            let a = get_next(&mut parts)??;
            let b = get_next(&mut parts)??;
            let c = get_next(&mut parts)??;
            (code, a, b, c)
        };

        let after = {
            let l = get_next(&mut lines)?;
            parse_reg(l)?
        };

        let sample = Sample { before, after, code, a, b, c };

        Ok(sample)
    }
}

fn parse_samples(s: &str) -> Result<Vec<Sample>, Box<dyn Error>> {
    s.trim().split("\n\n").map(Sample::from_str).collect()
}

fn parse_opcodes<F>(s: &str, mapping: F) -> Result<Vec<OpCode>, Box<dyn Error>>
where
    F: Fn(Val, Val, Val, Val) -> OpCode,
{
    fn parse_nums(l: &str) -> Result<(Val, Val, Val, Val), Box<dyn Error>> {
        let mut parts = l.split_whitespace().map(Val::from_str);
        let code = get_next(&mut parts)??;
        let a = get_next(&mut parts)??;
        let b = get_next(&mut parts)??;
        let c = get_next(&mut parts)??;
        let nums = (code, a, b, c);
        Ok(nums)
    }

    let mut program = Vec::new();
    for l in s.lines() {
        let (code, a, b, c) = parse_nums(l)?;
        let opcode = mapping(code, a, b, c);
        program.push(opcode);
    }
    Ok(program)
}

fn level1(samples: &[Sample]) -> usize {
    samples.iter().filter(|s| s.matching_opcodes().count() >= 3).count()
}

fn get_opcode_parser(
    samples: &[Sample],
) -> impl Fn(Val, Val, Val, Val) -> OpCode {
    let mut map = vec![HashSet::new(); 16];
    for sample in samples {
        let matching = sample
            .matching_opcodes()
            .map(|op| op.to_ident())
            .collect::<HashSet<&str>>();
        let candidate = map.get_mut(sample.code as usize).unwrap();
        if candidate.is_empty() {
            *candidate = matching;
        } else {
            *candidate = &*candidate & &matching;
        }
    }

    let mut seen = HashSet::new();
    loop {
        if let Some(j) = map
            .iter()
            .enumerate()
            .position(|(i, s)| s.len() == 1 && seen.insert(i))
        {
            for i in 0..map.len() {
                if i != j {
                    map[i] = &map[i] - &map[j];
                }
            }
        } else {
            break;
        }
    }

    let constructors = map
        .into_iter()
        .map(|ident_set| ident_set.into_iter().next().unwrap())
        .map(|ident| OpCode::get_constructor(ident).unwrap())
        .collect::<Vec<Box<dyn Fn(Val, Val, Val) -> OpCode>>>();

    move |op, a, b, c| constructors[op as usize](a, b, c)
}

fn level2(program: &[OpCode]) -> Val {
    let mut regs = Registers::new();
    for instr in program {
        regs.execute(*instr);
    }
    regs.0[0]
}

fn solve() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let parts = input.split("\n\n\n\n").collect::<Vec<&str>>();
    let samples = parse_samples(parts[0])?;

    let some = level1(&samples);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let opcode_mapper = get_opcode_parser(&samples);
    let program = parse_opcodes(parts[1], opcode_mapper)?;

    let thing = level2(&program);
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
        std::process::exit(1)
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");
    const EXAMPLE: &str = "
Before: [3, 2, 1, 1]
9 2 1 2
After:  [3, 2, 2, 1]";

    #[test]
    fn level1_examples() {
        let samples = parse_samples(EXAMPLE).unwrap();
        assert_eq!(level1(&samples), 1)
    }

    #[test]
    fn level1_regression() {
        let parts = INPUT.split("\n\n\n\n").collect::<Vec<&str>>();
        let samples = parse_samples(parts[0]).unwrap();

        assert_eq!(level1(&samples), 677);
    }

    #[test]
    fn level2_regression() {
        let parts = INPUT.split("\n\n\n\n").collect::<Vec<&str>>();
        let samples = parse_samples(parts[0]).unwrap();
        let opcode_mapper = get_opcode_parser(&samples);
        let program = parse_opcodes(parts[1], opcode_mapper).unwrap();

        assert_eq!(level2(&program), 540);
    }
}
