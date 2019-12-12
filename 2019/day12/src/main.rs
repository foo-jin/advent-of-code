use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

const STOP_TIME: u32 = 1000;

type Position = (i32, i32, i32);
type Velocity = (i32, i32, i32);

fn parse(s: &str) -> aoc::Result<Vec<(Position, Velocity)>> {
    s.trim()
        .lines()
        .map(|l| {
            let tr: &[_] = &['<', '>'];
            let mut parts = l.trim_matches(tr).split(", ");
            let x = parts
                .next()
                .ok_or_else(|| aoc::format_err!("Missing input"))?;
            let x = x.trim_start_matches("x=").parse()?;

            let y = parts
                .next()
                .ok_or_else(|| aoc::format_err!("Missing input"))?;
            let y = y.trim_start_matches("y=").parse()?;

            let z = parts
                .next()
                .ok_or_else(|| aoc::format_err!("Missing input"))?;
            let z = z.trim_start_matches("z=").parse()?;
            Ok(((x, y, z), (0, 0, 0)))
        })
        .collect()
}

fn level1(moons: &[(Position, Velocity)], stop_time: u32) -> aoc::Result<i32> {
    let mut moons = moons.to_owned();
    let n = moons.len();
    for t in 0..stop_time {
        if t % 10 == 0 {
            log::debug!("\nAfter {} steps:", t);
            for &((x, y, z), (dx, dy, dz)) in moons.iter() {
                log::debug!(
                    "pos=<x={}, y={}, z={}>, vel=<x={}, y={}, z={}>",
                    x,
                    y,
                    z,
                    dx,
                    dy,
                    dz
                );
            }
        }
        for i in 0..n {
            for j in i + 1..n {
                let ((x1, y1, z1), (dx1, dy1, dz1)) = moons[i];
                let ((x2, y2, z2), (dx2, dy2, dz2)) = moons[j];
                let (dx1, dx2) = adjust_velocity(x1, x2, dx1, dx2);
                let (dy1, dy2) = adjust_velocity(y1, y2, dy1, dy2);
                let (dz1, dz2) = adjust_velocity(z1, z2, dz1, dz2);
                moons[i] = ((x1, y1, z1), (dx1, dy1, dz1));
                moons[j] = ((x2, y2, z2), (dx2, dy2, dz2));
            }
        }

        for moon in moons.iter_mut() {
            let ((x, y, z), (dx, dy, dz)) = *moon;
            *moon = ((x + dx, y + dy, z + dz), (dx, dy, dz));
        }
    }

    let total_energy =
        moons.iter().map(|&(p1, p2)| abssum(p1) * abssum(p2)).sum();
    Ok(total_energy)
}

fn level2(init: &[(Position, Velocity)]) -> u64 {
    let mut moons = init.to_owned();
    let n = moons.len();

    let mut periods = HashMap::new();

    let mut t = 0;
    while periods.len() < 3 {
        // if t % 10000 == 0 {
        log::debug!("\nAfter {} steps:", t);
        for &((x, y, z), (dx, dy, dz)) in moons.iter() {
            log::debug!(
                "pos=<x={}, y={}, z={}>, vel=<x={}, y={}, z={}>",
                x,
                y,
                z,
                dx,
                dy,
                dz
            );
        }
        // }

        for i in 0..n {
            for j in i + 1..n {
                let ((x1, y1, z1), (dx1, dy1, dz1)) = moons[i];
                let ((x2, y2, z2), (dx2, dy2, dz2)) = moons[j];
                let (dx1, dx2) = adjust_velocity(x1, x2, dx1, dx2);
                let (dy1, dy2) = adjust_velocity(y1, y2, dy1, dy2);
                let (dz1, dz2) = adjust_velocity(z1, z2, dz1, dz2);
                moons[i] = ((x1, y1, z1), (dx1, dy1, dz1));
                moons[j] = ((x2, y2, z2), (dx2, dy2, dz2));
            }
        }

        for i in 0..n {
            let ((x, y, z), (dx, dy, dz)) = moons[i];
            moons[i] = ((x + dx, y + dy, z + dz), (dx, dy, dz));
        }

        t += 1;

        if moons.iter().all(|&(_p, (dx, _, _))| dx == 0) {
            periods.entry("x").or_insert(t);
        }
        if moons.iter().all(|&(_p, (_, dy, _))| dy == 0) {
            periods.entry("y").or_insert(t);
        }
        if moons.iter().all(|&(_p, (_, _, dz))| dz == 0) {
            periods.entry("z").or_insert(t);
        }
    }
    use num::Integer;
    periods.values().map(|&period| period * 2).fold(1, |acc, v| acc.lcm(&v))
}

fn adjust_velocity(a1: i32, a2: i32, da1: i32, da2: i32) -> (i32, i32) {
    let (da1, da2) = match (a1, a2) {
        _ if a1 > a2 => (da1 - 1, da2 + 1),
        _ if a1 < a2 => (da1 + 1, da2 - 1),
        _ => (da1, da2),
    };
    (da1, da2)
}

fn abssum((a, b, c): (i32, i32, i32)) -> i32 {
    a.abs() + b.abs() + c.abs()
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let parsed = parse(&input)?;

    let some = level1(&parsed, STOP_TIME)?;
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
            "<x=-8, y=-10, z=0>
<x=5, y=5, z=10>
<x=2, y=-7, z=3>
<x=9, y=-8, z=-3>",
        )?;
        let result = level1(&input, 100)?;
        assert_eq!(result, 1940);
        Ok(())
    }

    #[test_log::new]
    fn level2_examples() -> aoc::Result<()> {
        let input = parse(
            "<x=-1, y=0, z=2>
<x=2, y=-10, z=-7>
<x=4, y=-8, z=8>
<x=3, y=5, z=-1>",
        )?;
        let result = level2(&input);
        assert_eq!(result, 2772);

        let input = parse(
            "<x=-8, y=-10, z=0>
<x=5, y=5, z=10>
<x=2, y=-7, z=3>
<x=9, y=-8, z=-3>")?;
        let result = level2(&input);
        assert_eq!(result,4686774924);
        Ok(())
    }
}
