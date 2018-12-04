use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

#[derive(Clone, Copy, Debug)]
enum Action {
    BeginShift(usize),
    Sleep,
    WakeUp,
}

#[derive(Clone, Copy, Debug)]
struct Record {
    time: chrono::NaiveDateTime,
    action: Action,
}

impl std::str::FromStr for Record {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use failure::format_err;
        use lazy_static::lazy_static;
        use regex::Regex;

        lazy_static! {
            static ref DATE_RE: Regex =
                { Regex::new(r"\[(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})\]").unwrap() };
            static ref ID_RE: Regex = Regex::new(r"#(\d+)").unwrap();
        }

        let date_match = DATE_RE
            .find(s)
            .ok_or_else(|| format_err!("Unexpected input format"))?;

        let time = chrono::NaiveDateTime::parse_from_str(date_match.as_str(), "[%Y-%m-%d %H:%M]")?;
        let action = if s.contains("wakes up") {
            Action::WakeUp
        } else if s.contains("falls asleep") {
            Action::Sleep
        } else if s.contains("begins shift") {
            let caps = ID_RE
                .captures(s)
                .ok_or_else(|| format_err!("Unexpected input format"))?;
            let id = caps[1].parse()?;
            Action::BeginShift(id)
        } else {
            return Err(format_err!("Unexpected input format"))?;
        };

        let record = Record { time, action };
        Ok(record)
    }
}

fn parse_logs(s: &str) -> Result<Vec<Record>, failure::Error> {
    let mut logs = s
        .lines()
        .map(str::parse::<Record>)
        .collect::<Result<Vec<Record>, _>>()?;
    logs.sort_by_key(|l| l.time);
    Ok(logs)
}

/// Sorted input!
fn pick_guard<F>(logs: &[Record], strategy: F) -> u32
where
    F: Fn(&(usize, [u32; 60])) -> u32,
{
    use chrono::Timelike;

    let mut sleep_log: HashMap<usize, [u32; 60]> = HashMap::new();
    let mut guard = match logs.first().unwrap() {
        Record {
            action: Action::BeginShift(id),
            ..
        } => *id,
        _ => panic!("First action should always be a new guard shift"),
    };

    let mut sleep = None;

    for rec in logs {
        match rec.action {
            Action::BeginShift(id) => guard = id,
            Action::Sleep => sleep = Some(rec.time.minute()),
            Action::WakeUp => {
                let start = sleep.unwrap();
                let end = rec.time.minute();
                let guard_log = sleep_log.entry(guard).or_insert_with(|| [0; 60]);
                (start..end)
                    .into_iter()
                    .for_each(|i| guard_log[i as usize] += 1);
            }
        }
    }

    let (id, sleep) = sleep_log.into_iter().max_by_key(strategy).unwrap();
    let (minute, _count) = sleep
        .into_iter()
        .enumerate()
        .max_by_key(|(_i, x)| *x)
        .unwrap();
    id as u32 * minute as u32
}

fn level1(logs: &[Record]) -> u32 {
    pick_guard(logs, |(_k, v)| v.iter().sum::<u32>())
}

fn level2(logs: &[Record]) -> u32 {
    pick_guard(logs, |(_k, v)| *v.iter().max().unwrap())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let logs = parse_logs(&input)?;

    let strat1 = level1(&logs);
    writeln!(io::stderr(), "level 1: {}", strat1)?;

    let strat2 = level2(&logs);
    writeln!(io::stderr(), "level 2: {}", strat2)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", strat2)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");
    const EXAMPLE: &str = "[1518-11-01 00:00] Guard #10 begins shift
[1518-11-01 00:05] falls asleep
[1518-11-01 00:25] wakes up
[1518-11-01 00:30] falls asleep
[1518-11-01 00:55] wakes up
[1518-11-01 23:58] Guard #99 begins shift
[1518-11-02 00:40] falls asleep
[1518-11-02 00:50] wakes up
[1518-11-03 00:05] Guard #10 begins shift
[1518-11-03 00:24] falls asleep
[1518-11-03 00:29] wakes up
[1518-11-04 00:02] Guard #99 begins shift
[1518-11-04 00:36] falls asleep
[1518-11-04 00:46] wakes up
[1518-11-05 00:03] Guard #99 begins shift
[1518-11-05 00:45] falls asleep
[1518-11-05 00:55] wakes up";

    #[test]
    fn level1_examples() {
        let logs = parse_logs(EXAMPLE).unwrap();
        assert_eq!(level1(&logs), 240)
    }

    #[test]
    fn level2_examples() {
        let logs = parse_logs(EXAMPLE).unwrap();
        assert_eq!(level2(&logs), 4455)
    }

    #[test]
    fn level1_regression() {
        let logs = parse_logs(INPUT).unwrap();
        assert_eq!(level1(&logs), 146622);
    }

    #[test]
    fn level2_regression() {
        let logs = parse_logs(INPUT).unwrap();
        assert_eq!(level2(&logs), 31848);
    }
}
