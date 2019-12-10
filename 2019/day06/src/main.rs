use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::{self, Read},
};

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

fn parse(s: &str) -> HashMap<&str, Vec<&str>> {
    let mut graph = HashMap::new();
    for orbit in s.trim().lines() {
        let (center, obj) = {
            let mut parts = orbit.split(')');
            (parts.next().unwrap(), parts.next().unwrap())
        };

        graph.entry(center).or_insert_with(Vec::new).push(obj);
    }
    graph
}

fn level1(orbits: &HashMap<&str, Vec<&str>>) -> u32 {
    fn dfs(
        orbits: &HashMap<&str, Vec<&str>>,
        current: &str,
        depth: u32,
    ) -> u32 {
        let result = match orbits.get(current) {
            Some(children) =>
                children.iter().map(|c| dfs(orbits, c, depth + 1)).sum(),
            None => 0,
        };
        depth + result
    }

    dfs(orbits, "COM", 0)
}

fn level2(orbits: &HashMap<&str, Vec<&str>>) -> u32 {
    let undirected_orbits = {
        let mut g = HashMap::with_capacity(orbits.keys().len());
        for (&obj, children) in orbits {
            g.entry(obj).or_insert_with(Vec::new).extend(children);
            for &c in children {
                g.entry(c).or_insert_with(Vec::new).push(obj);
            }
        }
        g
    };

    let mut seen = HashSet::new();
    let mut q = VecDeque::new();
    q.push_back(("YOU", 0));

    while let Some((current, dist)) = q.pop_front() {
        if current == "SAN" {
            return dist - 2;
        }

        seen.insert(current);
        if let Some(children) = undirected_orbits.get(current) {
            q.extend(
                children
                    .iter()
                    .filter(|c| !seen.contains(*c))
                    .map(|&c| (c, dist + 1)),
            );
        }
    }
    panic!("Destination not found");
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    #[test]
    fn level1_examples() {
        let input = parse(
            "COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L",
        );
        assert_eq!(level1(&input), 42);
    }

    #[test]
    fn level2_examples() {
        let input = parse(
            "COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L
K)YOU
I)SAN",
        );
        assert_eq!(level2(&input), 4)
    }

    #[test]
    fn level1_sanity() {
        let parsed = parse(INPUT);
        assert_eq!(level1(&parsed), 194721);
    }

    #[test]
    fn level2_sanity() {
        let parsed = parse(INPUT);
        assert_eq!(level2(&parsed), 316);
    }
}
