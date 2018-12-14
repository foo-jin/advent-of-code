use std::{
    error::Error,
    io::{self, Read, Write},
};

struct CookBook {
    recipes: Vec<u8>,
    elfs: [usize; 2],
}

impl CookBook {
    fn new() -> Self {
        CookBook {
            recipes: vec![3, 7],
            elfs: [0, 1],
        }
    }

    fn make(&mut self) -> usize {
        let [a, b] = self.elfs;
        let new = self.recipes[a] + self.recipes[b];
        if new > 9 {
            self.recipes.push(new / 10)
        }
        self.recipes.push(new % 10);

        for elf in self.elfs.iter_mut() {
            *elf += 1 + self.recipes[*elf] as usize;
            *elf %= self.recipes.len();
        }

        self.recipes.len()
    }
}

fn level1(k: usize) -> u64 {
    let mut cookbook = CookBook::new();
    while cookbook.make() < k + 10 {}
    cookbook.recipes[k..k + 10]
        .iter()
        .cloned()
        .map(u64::from)
        .fold(0, |acc, x| (acc * 10) + x)
}

fn level2(k: &str) -> usize {
    let target = k
        .trim()
        .as_bytes()
        .iter()
        .map(|b| b - b'0')
        .collect::<Vec<u8>>();

    let mut cookbook = CookBook::new();
    let mut ptr = 0;

    loop {
        let n = cookbook.make();
        while ptr + target.len() < n {
            if target[..] == cookbook.recipes[ptr..ptr + target.len()] {
                return ptr;
            }

            ptr += 1;
        }
    }
}

fn solve() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let k = input.trim().parse()?;

    let some = level1(k);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&input);
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

    #[test]
    fn level1_examples() {
        assert_eq!(level1(9), 5_158_916_779);
        assert_eq!(level1(5), 0124515891);
        assert_eq!(level1(18), 9251071085);
        assert_eq!(level1(2018), 5941429882);
    }

    #[test]
    fn level2_examples() {
        assert_eq!(level2("51589"), 9);
        assert_eq!(level2("01245"), 5);
        assert_eq!(level2("92510"), 18);
        assert_eq!(level2("59414"), 2018);
    }

    #[test]
    fn level1_regression() {
        let input = INPUT.trim().parse().unwrap();
        assert_eq!(level1(input), 6107101544);
    }

    #[test]
    fn level2_regression() {
        assert_eq!(level2(INPUT), 20291131);
    }
}
