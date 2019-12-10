use std::io::{self, Read, Write};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let parsed = parse(&input);

    let some = level1(&parsed);
    eprintln!("level 1: {}", some);

    let thing = level2(&parsed);
    eprintln!("level 2: {}", thing);

    // stdout is used to submit solutions
    println!("{}", thing);
}

fn parse(s: &str) -> Vec<i32> {
    s.trim()
        .split(',')
        .map(str::parse::<i32>)
        .map(Result::unwrap)
        .collect::<Vec<i32>>()
}

fn level1(intcode: &[i32]) -> i32 {
    let mut intcode = intcode.to_owned();
    run_intcode(&mut intcode, &[1])
}

fn level2(intcode: &[i32]) -> i32 {
    let mut intcode = intcode.to_owned();
    run_intcode(&mut intcode, &[5])
}

fn run_intcode(intcode: &mut [i32], inputs: &[i32]) -> i32 {
    let mut ip = 0;
    let mut head = 0;
    let mut output = 0;
    loop {
        let mut instruction = intcode[ip];
        let opcode = (instruction % 100) as u8;
        instruction /= 100;
        let mut mode = [0u8; 3];
        for i in 0..3 {
            mode[i] = (instruction % 10) as u8;
            instruction /= 10;
        }

        match opcode {
            1 | 2 | 7 | 8 => {
                let mut args = [0; 2];
                for i in 0..2 {
                    let val = intcode[ip + i + 1];
                    args[i] = match mode[i] {
                        0 => intcode[val as usize],
                        1 => val,
                        _ => panic!("Unkown mode encountered: {}", mode[i]),
                    };
                }

                let out = intcode[ip + 3] as usize;
                intcode[out] = match opcode {
                    1 => args[0] + args[1],
                    2 => args[0] * args[1],
                    7 =>
                        if args[0] < args[1] {
                            1
                        } else {
                            0
                        },
                    8 =>
                        if args[0] == args[1] {
                            1
                        } else {
                            0
                        },
                    _ => unreachable!(),
                };

                ip += 4;
            },
            3 => {
                let address = intcode[ip + 1] as usize;
                intcode[address] = inputs[head];
                head += 1;
                ip += 2;
            },
            4 => {
                let address = intcode[ip + 1] as usize;
                let out = intcode[address];
                eprintln!("Output {}", out);
                output = out;
                ip += 2;
            },
            5 | 6 => {
                let mut args = [0; 2];
                for i in 0..2 {
                    let val = intcode[ip + i + 1];
                    args[i] = match mode[i] {
                        0 => intcode[val as usize],
                        1 => val,
                        _ => panic!("Unkown mode encountered: {}", mode[i]),
                    };
                }

                let b = match opcode {
                    5 => args[0] != 0,
                    6 => args[0] == 0,
                    _ => unreachable!(),
                };

                if b {
                    ip = args[1] as usize
                } else {
                    ip += 3;
                }
            },
            99 => return output,
            _ => panic!("Unknown opcode encountered: {}", opcode),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test]
    fn level1_examples() {
        let intcode = parse("3,0,4,0,99");
        assert_eq!(level1(&intcode), 1);

        let intcode = parse("1002,4,3,4,33");
        level1(&intcode);
    }

    #[test]
    fn level2_examples() {
        let mut intcode = parse("3,9,8,9,10,9,4,9,99,-1,8");
        assert_eq!(run_intcode(&mut intcode, &[8]), 1);

        let mut intcode = parse("3,9,8,9,10,9,4,9,99,-1,8");
        assert_eq!(run_intcode(&mut intcode, &[7]), 0);

        let mut intcode = parse("3,9,7,9,10,9,4,9,99,-1,8");
        assert_eq!(run_intcode(&mut intcode, &[7]), 1);

        let mut intcode = parse("3,9,7,9,10,9,4,9,99,-1,8");
        assert_eq!(run_intcode(&mut intcode, &[8]), 0);

        let mut intcode = parse("3,9,7,9,10,9,4,9,99,-1,8");
        assert_eq!(run_intcode(&mut intcode, &[8]), 0);
    }

    #[test]
    fn level1_sanity() {
        let parsed = parse(INPUT);
        assert_eq!(level1(&parsed), 4887191);
    }

    #[test]
    fn level2_sanity() {
        let parsed = parse(INPUT);
        assert_eq!(level2(&parsed), 3419022);
    }
}
