use std::{
    convert::TryFrom,
    fmt,
    io::{self, Read, Write},
};

macro_rules! format_err {
    ($($tt:tt)*) => { Box::<dyn std::error::Error>::from(format!($($tt)*)) }
}

mod aoc {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}

const IMAGE_WIDTH: usize = 25;
const IMAGE_HEIGHT: usize = 6;

struct Image([u8; IMAGE_WIDTH * IMAGE_HEIGHT]);

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for row in self.0.chunks(IMAGE_WIDTH) {
            if first {
                first = false;
            } else {
                write!(f, "{}", '\n')?;
            }

            for x in row {
                let c = match x {
                    0 => '░',
                    1 => '▓',
                    2 => ' ',
                    _ => unreachable!(),
                };
                write!(f, "{}", c)?;
            }
        }

        Ok(())
    }
}

fn parse(s: &str) -> aoc::Result<Vec<u8>> {
    s.trim()
        .chars()
        .map(|c| {
            c.to_digit(10)
                .ok_or_else(|| {
                    format_err!("Input contained a non-digit character")
                })
                .and_then(|d| u8::try_from(d).map_err(Into::into))
        })
        .collect()
}

fn level1(image: &[u8]) -> aoc::Result<u32> {
    let mut result = None;
    for layer in image.chunks(IMAGE_WIDTH * IMAGE_HEIGHT) {
        let mut counts = [0u8; 10];
        for &x in layer {
            counts[x as usize] += 1;
        }

        let res = Some((counts[0], counts[1] as u32 * counts[2] as u32));
        result = match result {
            Some((zeros, _)) if counts[0] < zeros => res,
            None => res,
            _ => result,
        }
    }

    result.ok_or_else(|| format_err!("No result found")).map(|(_, x)| x)
}

fn level2(image: &[u8]) -> Image {
    let mut final_image = [2u8; IMAGE_WIDTH * IMAGE_HEIGHT];
    for layer in image.chunks(IMAGE_WIDTH * IMAGE_HEIGHT) {
        for (&p, pixel) in layer.into_iter().zip(final_image.iter_mut()) {
            if *pixel == 2 {
                *pixel = p;
            }
        }
    }

    Image(final_image)
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let parsed = parse(&input)?;

    let some = level1(&parsed)?;
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&parsed);
    writeln!(io::stderr(), "level 2:\n{}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", some)?;
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
    fn sanity() -> aoc::Result<()> {
        let input = parse(INPUT)?;
        let result = level1(&input)?;
        assert_eq!(result, 1330);

        let expected = "\
▓▓▓▓░░▓▓░░▓░░▓░▓▓▓▓░▓▓▓▓░
▓░░░░▓░░▓░▓░░▓░▓░░░░▓░░░░
▓▓▓░░▓░░▓░▓▓▓▓░▓▓▓░░▓▓▓░░
▓░░░░▓▓▓▓░▓░░▓░▓░░░░▓░░░░
▓░░░░▓░░▓░▓░░▓░▓░░░░▓░░░░
▓░░░░▓░░▓░▓░░▓░▓▓▓▓░▓░░░░";
        let result = format!("{}", level2(&input));
        assert_eq!(result, expected);
        Ok(())
    }
}
