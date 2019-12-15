use regex::Regex;
use std::{
    collections::BTreeMap as Map,
    io::{self, Read, Write},
};

type Resource<'a> = (&'a str, u64);
type ObligationMap<'a> = Map<&'a str, (Vec<Resource<'a>>, u64)>;

fn parse<'a>(s: &'a str) -> aoc::Result<ObligationMap<'a>> {
    let re = Regex::new(r"(\d+) ([[:alpha:]]+)")?;
    s.trim()
        .lines()
        .map(|l| {
            let mut parts = l.split("=>");
            let first = parts
                .next()
                .ok_or_else(|| aoc::format_err!("Missing reaction input"))?;
            let second = parts
                .next()
                .ok_or_else(|| aoc::format_err!("Missing reaction output"))?;
            let mut obligations = Vec::new();
            for cap in re.captures_iter(first) {
                let amount = cap[1].parse::<u64>()?;
                obligations.push((cap.get(2).unwrap().as_str(), amount));
            }
            if obligations.len() == 0 {
                return aoc::err!("No resources found in reaction input");
            }
            let cap = re.captures(second).ok_or_else(|| {
                aoc::format_err!("No resource found in reacion output")
            })?;
            let amount = cap[1].parse::<u64>()?;
            let id = cap.get(2).unwrap().as_str();
            Ok((id, (obligations, amount)))
        })
        .collect()
}

fn level1<'a>(reactions: &ObligationMap) -> u64 {
    ore_required(reactions, 1)
}

fn level2<'a>(reactions: &ObligationMap) -> u64 {
    const ORE: u64 = 1000000000000;
    let mut fuel = 1;
    let mut needed;
    loop {
        needed = ore_required(reactions, fuel + 1);
        if needed > ORE {
            return fuel;
        } else {
            fuel = u64::max(fuel + 1, (fuel + 1) * ORE / needed)
        }
    }
}

fn ore_required(reactions: &ObligationMap, target: u64) -> u64 {
    let mut available = Map::new();
    let mut missing = vec![("FUEL", target)];
    let mut ore = 0;
    while let Some((material, mut quantity)) = missing.pop() {
        if let Some(a) = available.get_mut(material) {
            let consumed = quantity.min(*a);
            *a -= consumed;
            quantity -= consumed;
        }

        if quantity > 0 {
            let (reqs, produced) = &reactions[material];
            let reps = (quantity + produced - 1) / produced;
            let added = produced * reps - quantity;
            available.insert(material, added);
            for (mat, k) in reqs {
                match *mat {
                    "ORE" => ore += k * reps,
                    _ => missing.push((mat, k * reps)),
                }
            }
        }
    }

    ore
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let parsed = parse(&input)?;

    let some = level1(&parsed);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&parsed);
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
        let input = parse(
            "10 ORE => 10 A
1 ORE => 1 B
7 A, 1 B => 1 C
7 A, 1 C => 1 D
7 A, 1 D => 1 E
7 A, 1 E => 1 FUEL",
        )?;
        let result = level1(&input);
        assert_eq!(result, 31, "example 1");

        let input = parse(
            "171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX",
        )?;
        let result = level1(&input);
        assert_eq!(result, 2210736, "example 5");

        Ok(())
    }
}
