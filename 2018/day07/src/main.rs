use petgraph::prelude::*;
use std::{
    cmp,
    collections::BinaryHeap,
    io::{self, Read, Write},
};

const WORKERS: u32 = 5;
const BASE_DURATION: u32 = 60;

type Step = u8;
type Graph = DiGraphMap<Step, (Step, Step)>;
type MinHeap<T> = BinaryHeap<cmp::Reverse<T>>;

struct World {
    step_graph: Graph,
    queue: MinHeap<Step>,
    schedule: MinHeap<(u32, Step)>,
    workers_available: u32,
    base_duration: u32,
    current_time: u32,
}

impl World {
    fn new(step_graph: Graph, workers: u32, base_duration: u32) -> Self {
        let queue = root_graph(&step_graph)
            .map(cmp::Reverse)
            .collect::<MinHeap<Step>>();
        World {
            step_graph,
            workers_available: workers,
            base_duration,
            current_time: 0,
            queue,
            schedule: MinHeap::new(),
        }
    }

    fn take_jobs(&mut self) {
        while self.workers_available > 0 && !self.queue.is_empty() {
            let n = self.queue.pop().unwrap().0;
            self.workers_available -= 1;
            let dt = n - b'A' + 1;
            let dt = self.base_duration + dt as u32;
            self.schedule
                .push(cmp::Reverse((self.current_time + dt, n)));
        }
    }

    fn fast_forward(&mut self) {
        if let Some((tn, n)) = self.schedule.pop().map(|r| r.0) {
            debug_assert!(self.current_time <= tn);
            self.current_time = tn;
            self.workers_available += 1;
            remove_root_node(&mut self.step_graph, &mut self.queue, n);
        }
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

fn parse_graph(s: &str) -> Graph {
    DiGraphMap::from_edges(s.trim().lines().map(|l| {
        let mut parts = l.split_whitespace();
        let a = parts.nth(1).unwrap().as_bytes()[0];
        let b = parts.nth(5).unwrap().as_bytes()[0];
        (a, b)
    }))
}

fn root_graph<'a>(graph: &'a Graph) -> impl Iterator<Item = Step> + 'a {
    graph.nodes().filter(move |n| {
        graph
            .neighbors_directed(*n, Direction::Incoming)
            .next()
            .is_none()
    })
}

fn remove_root_node(graph: &mut Graph, root_nodes: &mut MinHeap<Step>, n: Step) {
    for m in graph.neighbors_directed(n, Direction::Outgoing) {
        if graph.neighbors_directed(m, Direction::Incoming).count() == 1 {
            root_nodes.push(cmp::Reverse(m));
        }
    }

    graph.remove_node(n);
}

fn level1(mut graph: Graph) -> String {
    let mut queue = root_graph(&graph)
        .map(cmp::Reverse)
        .collect::<MinHeap<Step>>();
    let mut ordering = Vec::new();

    while let Some(n) = queue.pop().map(|r| r.0) {
        ordering.push(n);
        remove_root_node(&mut graph, &mut queue, n);
    }

    unsafe { String::from_utf8_unchecked(ordering) }
}

fn level2(graph: Graph, workers: u32, base_duration: u32) -> u32 {
    let mut world = World::new(graph, workers, base_duration);
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
