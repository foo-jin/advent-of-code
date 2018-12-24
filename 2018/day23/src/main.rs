use std::{
    error::Error,
    io::{self, Read, Write},
    str::FromStr,
};

macro_rules! format_err {
    ($($tt:tt)*) => { Box::<std::error::Error>::from(format!($($tt)*)) }
}

mod aoc {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let bots = parse_bots(&input)?;

    let some = level1(&bots);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&bots);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

fn parse_bots(s: &str) -> aoc::Result<Box<[NanoBot]>> {
    s.trim().lines().map(|l| l.parse()).collect()
}

fn level1(bots: &[NanoBot]) -> usize {
    let strongest = bots.iter().max_by_key(|b| b.rd).unwrap();
    bots.iter().filter(|b| strongest.contains(b.pos)).count()
}

fn level2(bots: &[NanoBot]) -> u32 {
    fn abs<'ctx>(ctx: &'ctx z3::Context, a: &z3::Ast<'ctx>) -> z3::Ast<'ctx> {
        a.ge(&ctx.from_u64(0)).ite(a, &a.minus())
    }

    fn absdiff<'ctx>(
        ctx: &'ctx z3::Context,
        a: &z3::Ast<'ctx>,
        b: &z3::Ast<'ctx>,
    ) -> z3::Ast<'ctx> {
        abs(ctx, &a.sub(&[b]))
    }

    let cfg = z3::Config::new();
    let ctx = z3::Context::new(&cfg);
    let opt = z3::Optimize::new(&ctx);

    let zero = ctx.from_u64(0);
    let one = ctx.from_u64(1);

    let x = ctx.named_int_const("x");
    let y = ctx.named_int_const("y");
    let z = ctx.named_int_const("z");
    // dist := x + y + z
    let dist = ctx.named_int_const("dist");
    opt.assert(&dist._eq(&abs(&ctx, &x).add(&[&abs(&ctx, &y), &abs(&ctx, &z)])));

    let in_range: Vec<z3::Ast> = (0..bots.len() as u32)
        .map(|i| ctx.numbered_int_const(i))
        .collect();

    for (var, bot) in in_range.iter().zip(bots.iter()) {
        let dx = &absdiff(&ctx, &x, &ctx.from_i64(bot.pos[0]));
        let dy = &absdiff(&ctx, &y, &ctx.from_i64(bot.pos[1]));
        let dz = &absdiff(&ctx, &z, &ctx.from_i64(bot.pos[2]));
        let rd = &ctx.from_u64(bot.rd as u64);
        let bot_dist = &zero.add(&[dx, dy, dz]);
        opt.assert(&var._eq(&bot_dist.le(rd).ite(&one, &zero)));
    }

    // overlap := |{ b \in bots | b.contains(<x, y, z>) }|
    let overlap = ctx.named_int_const("overlap");
    let in_range_refs: Vec<&z3::Ast> = in_range.iter().collect();
    opt.assert(&overlap._eq(&zero.add(&in_range_refs)));

    opt.maximize(&overlap);
    opt.minimize(&dist);
    assert!(opt.check());

    fn get_val<'ctx>(model: &z3::Model<'ctx>, a: &z3::Ast<'ctx>) -> i64 {
        model.eval(a).and_then(|v| v.as_i64()).unwrap()
    }

    let model = opt.get_model();
    let xv = get_val(&model, &x);
    let yv = get_val(&model, &y);
    let zv = get_val(&model, &z);
    let dist = model.eval(&dist).and_then(|v| v.as_i64()).unwrap();
    log::info!("xyz: <{}, {}, {}>, dist: {}", xv, yv, zv, dist);
    dist as u32
}

fn absdiff(a: i64, b: i64) -> u32 {
    (a - b).abs() as u32
}

fn manhattan(p: Point, q: Point) -> u32 {
    p.iter().zip(q.iter()).map(|(&a, &b)| absdiff(a, b)).sum()
}

type Point = [i64; 3];

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct NanoBot {
    pos: Point,
    rd: u32,
}

impl NanoBot {
    fn contains(&self, p: Point) -> bool {
        manhattan(self.pos, p) <= self.rd
    }
}

impl FromStr for NanoBot {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use regex::Regex;
        lazy_static::lazy_static! {
            static ref RE: Regex =
                Regex::new(
                    r"pos=<(?P<x>[-+]?[0-9]+),(?P<y>[-+]?[0-9]+),(?P<z>[-+]?[0-9]+)>,\sr=(?P<r>[0-9]+)"
                ).unwrap();
        }
        let caps = RE
            .captures(s)
            .ok_or_else(|| format_err!("invalid input format: {}", s))?;
        let pos = [caps["x"].parse()?, caps["y"].parse()?, caps["z"].parse()?];
        let rd = caps["r"].parse()?;
        Ok(NanoBot { pos, rd })
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

    #[test_log::new]
    fn regression() -> aoc::Result<()> {
        let bots = parse_bots(INPUT)?;
        assert_eq!(level1(&bots), 232, "level 1 regressed");
        assert_eq!(level2(&bots), 82010396, "level 2 regressed");
        Ok(())
    }

    const EX1: &str = "
pos=<0,0,0>, r=4
pos=<1,0,0>, r=1
pos=<4,0,0>, r=3
pos=<0,2,0>, r=1
pos=<0,5,0>, r=3
pos=<0,0,3>, r=1
pos=<1,1,1>, r=1
pos=<1,1,2>, r=1
pos=<1,3,1>, r=1";

    #[test_log::new]
    fn level1_examples() -> aoc::Result<()> {
        let bots = parse_bots(EX1)?;
        assert_eq!(level1(&bots), 7);
        Ok(())
    }

    const EX2: &str = "
pos=<10,12,12>, r=2
pos=<12,14,12>, r=2
pos=<16,12,12>, r=4
pos=<14,14,14>, r=6
pos=<50,50,50>, r=200
pos=<10,10,10>, r=5";

    #[test_log::new]
    fn level2_examples() -> aoc::Result<()> {
        let bots = parse_bots(EX2)?;
        assert_eq!(level2(&bots), 36);
        Ok(())
    }
}