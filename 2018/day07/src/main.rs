use petgraph::prelude::*;
use std::{
    collections::BTreeSet,
    io::{self, Read, Write},
};

const WORKERS: u32 = 5;
const BASE_DURATION: u32 = 60;

type Graph<N, E> = DiGraphMap<N, E>;

struct State {
    step_graph: Graph<u8, (u8, u8)>,
    queue: BTreeSet<u8>,
    schedule: BTreeSet<(u32, u8)>,
    workers_available: u32,
    base_duration: u32,
    current_time: u32,
}

impl State {
    fn new(step_graph: Graph<u8, (u8, u8)>, workers: u32, base_duration: u32) -> Self {
        let queue = root_graph(&step_graph).collect::<BTreeSet<u8>>();
        State {
            step_graph,
            workers_available: workers,
            base_duration,
            current_time: 0,
            queue,
            schedule: BTreeSet::new(),
        }
    }

    fn take_jobs(&mut self) {
        while self.workers_available > 0 && !self.queue.is_empty() {
            let n = *self.queue.iter().next().unwrap();
            self.queue.remove(&n);
            self.workers_available -= 1;
            let dt = n - b'A' + 1;
            let dt = self.base_duration + dt as u32;
            self.schedule.insert((self.current_time + dt, n));
        }
    }

    fn fast_forward(&mut self) {
        let (tn, n) = *self.schedule.iter().next().unwrap();
        debug_assert!(self.current_time <= tn);
        self.schedule.remove(&(tn, n));
        self.current_time = tn;
        self.workers_available += 1;
        remove_root_node(&mut self.step_graph, &mut self.queue, n);
    }

    fn done_processing(&self) -> bool {
        self.queue.is_empty() && self.schedule.is_empty()
    }

    fn simulate_to_end(&mut self) {
        while !self.done_processing() {
            self.take_jobs();
            self.fast_forward();
        }
    }
}


fn parse_graph(s: &str) -> Graph<u8, (u8, u8)> {
    DiGraphMap::from_edges(s.trim().lines().map(|l| {
        let mut parts = l.split_whitespace();
        let a = parts.nth(1).unwrap().as_bytes()[0];
        let b = parts.nth(5).unwrap().as_bytes()[0];
        (a, b)
    }))
}

fn root_graph<'a, N, E>(graph: &'a Graph<N, E>) -> impl Iterator<Item = N> + 'a
where
    N: petgraph::graphmap::NodeTrait,
{
    graph.nodes().filter(move |n| {
        graph
            .neighbors_directed(*n, Direction::Incoming)
            .next()
            .is_none()
    })
}

fn remove_root_node<N, E>(graph: &mut Graph<N, E>, root_nodes: &mut BTreeSet<N>, n: N)
where
    N: petgraph::graphmap::NodeTrait,
{
    for m in graph.neighbors_directed(n, Direction::Outgoing) {
        if graph.neighbors_directed(m, Direction::Incoming).count() == 1 {
            root_nodes.insert(m);
        }
    }

    graph.remove_node(n);
}

fn level1(mut graph: Graph<u8, (u8, u8)>) -> String {
    let mut queue = root_graph(&graph).collect::<BTreeSet<u8>>();
    let mut ordering = Vec::new();

    while let Some(&n) = queue.iter().next() {
        queue.remove(&n);
        ordering.push(n);
        remove_root_node(&mut graph, &mut queue, n);
    }

    unsafe { String::from_utf8_unchecked(ordering) }
}

fn level2(graph: Graph<u8, (u8, u8)>, workers: u32, base_duration: u32) -> u32 {
    let mut world = State::new(graph, workers, base_duration);
    world.simulate_to_end();
    world.current_time
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let dependencies = parse_graph(&input);

    let some = level1(dependencies.clone());
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(dependencies, WORKERS, BASE_DURATION);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", some)?;
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
        let graph = parse_graph(EXAMPLE);
        assert_eq!(level1(graph), "CABDFE");
    }

    #[test]
    fn level2_examples() {
        let graph = parse_graph(EXAMPLE);
        assert_eq!(level2(graph, 2, 0), 15);
    }

    #[test]
    fn level1_regression() {
        let graph = parse_graph(INPUT);
        assert_eq!(level1(graph), "BGKDMJCNEQRSTUZWHYLPAFIVXO".to_string());
    }

    #[test]
    fn level2_regression() {
        let graph = parse_graph(INPUT);
        assert_eq!(level2(graph, WORKERS, BASE_DURATION), 941);
    }
}
