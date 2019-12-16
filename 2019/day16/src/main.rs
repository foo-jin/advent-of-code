use std::io::{self, Read, Write};

const PHASES: u8 = 100;
const PATTERN: [i64; 4] = [0, 1, 0, -1];

fn parse(s: &str) -> aoc::Result<Vec<i64>> {
    s.trim()
        .chars()
        .map(|c| c.to_digit(10).map(|d| d as i64))
        .collect::<Option<Vec<i64>>>()
        .ok_or_else(|| aoc::format_err!("Non-decimal digit encountered"))
}

fn level1(input_list: &[i64], phases: u8) -> aoc::Result<String> {
    let mut output_list = input_list.to_vec();
    let n = output_list.len();

    for _phase in 0..phases {
        output_list = (0..n)
            .map(|i| {
                let mut pat = PATTERN.iter().cloned().cycle();
                let mut a = pat.next().unwrap();
                let mut j = i;
                let mut sum = 0;
                while j < n {
                    if (j + 1) % (i + 1) == 0 {
                        a = pat.next().unwrap()
                    }

                    sum += output_list[j] * a;
                    j += 1;
                }
                ((sum % 10).abs())
            })
            .collect();
    }

    let mut result = String::new();
    for x in output_list.into_iter().take(8) {
        result.push_str(&x.to_string());
    }

    Ok(result)
}

fn level2(input_list: &[i64], phases: u8) -> aoc::Result<String> {
    let mut output_list = input_list
        .iter()
        .cloned()
        .cycle()
        .take(input_list.len() * 10_000)
        .collect::<Vec<i64>>();

    let offset = {
        output_list
            .iter()
            .cloned()
            .take(7)
            .map(|x| x as usize)
            .fold(0, |acc, x| acc * 10 + x)
    };
    dbg!(offset);

    for _phase in 0..phases {
        let mut partial_sum = output_list[offset..].iter().sum::<i64>();
        for x in output_list[offset..].iter_mut() {
            let tmp = partial_sum;
            partial_sum -= *x;
            *x = (tmp % 10).abs()
        }
    }

    let mut result = String::new();
    for x in output_list.into_iter().skip(offset).take(8) {
        result.push_str(&x.to_string());
    }

    Ok(result)
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let parsed = parse(&input)?;

    let some = level1(&parsed, PHASES)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&parsed, PHASES)?;
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
        let input = parse("12345678")?;
        let result = level1(&input, 4)?;
        assert_eq!(&result, "01029498", "ex 1");

        let input = parse("80871224585914546619083218645595")?;
        let result = level1(&input, PHASES)?;
        assert_eq!(&result, "24176176", "ex 2");

        let input = parse("19617804207202209144916044189917")?;
        let result = level1(&input, PHASES)?;
        assert_eq!(&result, "73745418", "ex 3");

        let input = parse("69317163492948606335995924319873")?;
        let result = level1(&input, PHASES)?;
        assert_eq!(&result, "52432133", "ex 4");

        Ok(())
    }
}
