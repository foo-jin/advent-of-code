#![feature(vec_remove_item)]
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    io::{self, Read, Write},
};

fn parse_graph<F, K, V>(s: &str, f: F) -> HashMap<K, Vec<V>>
where
    F: Fn((u8, u8)) -> (K, V),
    K: Eq + std::hash::Hash,
{
    use itertools::Itertools;

    s.trim()
        .lines()
        .map(|l| {
            let mut parts = l.split_whitespace();
            let a = parts.nth(1).unwrap().as_bytes()[0];
            let b = parts.nth(5).unwrap().as_bytes()[0];
            f((a, b))
        })
        .into_group_map()
}

fn level1(s: &str) -> String {
    let mut graph = parse_graph(s, |p| p);
    let mut rev_graph = parse_graph(s, |(a, b)| (b, a));

    let keys = graph.keys().cloned().collect::<HashSet<u8>>();
    let vals = rev_graph.keys().cloned().collect();
    let mut root = keys.difference(&vals).cloned().collect::<BTreeSet<u8>>();

    for v in vals.difference(&keys) {
        graph.entry(*v).or_default();
    }

    let mut dag = Vec::new();
    while let Some(&n) = root.iter().next() {
        root.remove(&n);
        dag.push(n);
        for &m in graph.get(&n).unwrap() {
            let incoming = rev_graph.entry(m).or_default();
            incoming.remove_item(&n);
            if incoming.is_empty() {
                root.insert(m);
            }
        }
        graph.remove(&n);
    }
    unsafe { String::from_utf8_unchecked(dag) }
}

fn level2(s: &str, workers: u32, base_duration: u32) -> u32 {
    let mut graph = parse_graph(s, |p| p);
    let mut rev_graph = parse_graph(s, |(a, b)| (b, a));

    let keys = graph.keys().cloned().collect::<HashSet<_>>();
    let vals = rev_graph.keys().cloned().collect();
    let mut queue = keys.difference(&vals).cloned().collect::<BTreeSet<_>>();

    for v in vals.difference(&keys) {
        graph.entry(*v).or_default();
    }

    let mut t = 0;
    let mut workers = workers;
    let mut scheduled = BTreeSet::new();

    while !queue.is_empty() || !scheduled.is_empty() {
        eprintln!(
            "t: {}\nworkers: {}\nqueue: {:?}\nscheduled: {:?}",
            t, workers, queue, scheduled
        );
        while workers > 0 && !queue.is_empty() {
            let n = *queue.iter().next().unwrap();
            queue.remove(&n);
            workers -= 1;
            let dt = n - b'A' + 1;
            scheduled.insert((t + base_duration + dt as u32, n));
        }

        let (tn, n) = *scheduled.iter().next().unwrap();
        scheduled.remove(&(tn, n));
        t = tn;
        workers += 1;
        eprintln!("n: {}", n as char);
        for &m in graph.get(&n).unwrap() {
            eprintln!("m: {}", m as char);
            let incoming = rev_graph.entry(m).or_default();
            incoming.remove_item(&n);
            if incoming.is_empty() {
                queue.insert(m);
            }
        }
        graph.remove(&n);
    }

    t
}

const WORKERS: u32 = 5;
const BASE_DURATION: u32 = 60;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // let some = level1(&input);
    // writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&input, WORKERS, BASE_DURATION);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");
    const EXAMPLE: &str = "Step C must be finished before step A can begin.
Step C must be finished before step F can begin.
Step A must be finished before step B can begin.
Step A must be finished before step D can begin.
Step B must be finished before step E can begin.
Step D must be finished before step E can begin.
Step F must be finished before step E can begin.";

    #[test]
    fn level1_examples() {
        assert_eq!(level1(EXAMPLE), "CABDFE");
    }

    #[test]
    fn level2_examples() {
        assert_eq!(level2(EXAMPLE, 2, 0), 15);
    }

    #[test]
    fn level1_regression() {
        assert_eq!(level1(INPUT), "BGKDMJCNEQRSTUZWHYLPAFIVXO".to_string());
    }

    #[test]
    fn level2_regression() {
        assert_eq!(level2(INPUT, WORKERS, BASE_DURATION), 941);
    }
}
