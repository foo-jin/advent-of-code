const INPUT: &str = include_str!("../input.txt");

#[allow(dead_code)]
fn part1(s: &str) -> i32 {
    s.lines().map(|s| s.parse::<i32>().unwrap()).sum()
}

fn part2(s: &str) -> i32 {
    use std::collections::HashSet;

    let mut freq = 0i32;
    let mut seen = HashSet::new();
    seen.insert(0);

    for x in s.lines().map(|s| s.parse::<i32>().unwrap()).cycle() {
        freq += x;
        if !seen.insert(freq) {
            break;
        }
    }

    freq
}

fn main() -> Result<(), failure::Error> {
    // let freq = part1(INPUT);
    let freq = part2(INPUT);
    print!("{}", freq);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_first_twice(input: &str, expected: i32) {
        let result = part2(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn first_twice() {
        let input = "+1\n-1";
        check_first_twice(input, 0);

        let input = "+3\n+3\n+4\n-2\n-4";
        check_first_twice(input, 10);

        let input = "-6\n+3\n+8\n+5\n-6";
        check_first_twice(input, 5);

        let input = "+7\n+7\n-2\n-7\n-4";
        check_first_twice(input, 14);
    }

    #[test]
    fn part1_regresssion() {
        assert_eq!(part1(INPUT), 547);
    }

    #[test]
    fn part2_regresssion() {
        assert_eq!(part2(INPUT), 76414);
    }
}
