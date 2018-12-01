use std::{
    collections::HashSet,
    io::{self, BufRead, Read},
};

fn get_input() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn _part1<R: BufRead>(read: R) -> i32 {
    read.lines()
        .map(Result::unwrap)
        .map(|s| s.parse::<i32>())
        .map(Result::unwrap)
        .sum()
}

fn part2(changes: &[i32]) -> i32 {
    let mut seen = HashSet::with_capacity(changes.len());
    seen.insert(0);
    let mut total = 0i32;

    for x in changes.into_iter().cycle() {
        total += x;
        if !seen.insert(total) {
            break
        }
    }

    total
}

fn main() -> Result<(), failure::Error> {
    let changes = get_input()?
        .lines()
        .map(|s| s.parse::<i32>())
        .collect::<Result<Vec<i32>, _>>()?;

    let total = part2(&changes);

    print!("{}", total);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_first_twice(input: &[i32], expected: i32) {
        let result = part2(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn first_twice() {
        let input = vec![1, -1];
        check_first_twice(&input, 0);

        let input = vec![3, 3, 4, -2, -4];
        check_first_twice(&input, 10);

        let input = vec![-6, 3, 8, 5, -6];
        check_first_twice(&input, 5);

        let input = vec![7, 7, -2, -7, -4];
        check_first_twice(&input, 14);
    }
}
