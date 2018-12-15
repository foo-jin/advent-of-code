use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
    fmt,
    io::{self, Read, Write},
    ops, str,
};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Point = [usize; 2];

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Wall,
    Free,
    Taken(UnitID),
}

impl State {
    fn is_free(&self) -> bool {
        *self == State::Free
    }

    fn as_unit(&self) -> Option<UnitID> {
        match self {
            State::Taken(u) => Some(*u),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct World {
    dimensions: Point,
    grid: HashMap<Point, State>,
    round: u32,
    units: Vec<UnitID>,
    em: EntityManager,
}

impl World {
    fn set_em(&mut self, em: EntityManager) {
        self.em = em;
    }

    fn game_over(&self) -> bool {
        self.em
            .alive
            .iter()
            .filter(|(_s, count)| **count > 0)
            .count()
            == 1
    }

    fn remaining_hp(&self) -> u32 {
        self.em
            .units
            .iter()
            .flatten()
            .map(|u| u32::from(u.hp))
            .sum()
    }

    fn neighbours(p: Point) -> impl Iterator<Item = Point> {
        use std::iter;
        let [x, y] = p;

        iter::once([x, y.saturating_sub(1)])
            .chain(iter::once([x.saturating_sub(1), y]))
            .chain(iter::once([x + 1, y]))
            .chain(iter::once([x, y + 1]))
    }

    fn free_neighbours(grid: &HashMap<Point, State>, p: Point) -> impl Iterator<Item = Point> + '_ {
        World::neighbours(p).filter(move |p| {
            let state = grid.get(p);
            state.map(|s| s.is_free()).unwrap_or(false)
        })
    }

    fn enemy_neighbours<'a>(
        grid: &'a HashMap<Point, State>,
        em: &'a EntityManager,
        unit: Unit,
    ) -> impl Iterator<Item = Unit> + 'a {
        World::neighbours(unit.pos)
            .flat_map(move |p| grid.get(&p))
            .filter_map(State::as_unit)
            .map(move |id| em[id].expect("dead unit shouldnt be in grid"))
            .filter(move |u| u.species != unit.species)
    }

    fn evolve(self) -> Self {
        fn attack(grid: &mut HashMap<Point, State>, em: &mut EntityManager, unit: Unit) {
            let target = World::enemy_neighbours(&grid, &em, unit).min_by_key(|u| u.hp);

            if let Some(mut target) = target {
                // eprintln!(
                //     "Combat occurs! {}{} is attacking {}{}",
                //     unit.species, unit.id, target.species, target.id
                // );
                target.hp = target.hp.saturating_sub(u16::from(unit.ap));
                em.update(target);
                if em[target.id].is_none() {
                    *grid.get_mut(&target.pos).unwrap() = State::Free;
                }
            }
        }

        let World {
            dimensions,
            mut units,
            mut grid,
            mut round,
            mut em,
        } = self;
        units = units.drain(..).filter(|u| !em[*u].is_none()).collect();
        units.sort_by_key(|u| em[*u].map(|u| u.pos).map(|[x, y]| [y, x]));

        for u in &units {
            if em.alive.iter().filter(|(_s, count)| **count > 0).count() == 1 {
                return World {
                    dimensions,
                    units,
                    grid,
                    round,
                    em,
                };
            }
            let mut unit = match em[*u] {
                Some(unit) => unit,
                None => continue,
            };

            if World::enemy_neighbours(&grid, &em, unit).count() > 0 {
                // eprintln!(
                //     "unit {}{} is already in range of an enemy",
                //     unit.species, unit.id,
                // );
                attack(&mut grid, &mut em, unit);
                continue;
            }

            let targets = em
                .get_enemies(unit.species)
                .flat_map(|u| World::free_neighbours(&grid, u.pos))
                .collect::<HashSet<Point>>();

            let mut encountered = HashSet::new();
            let mut generation = 0;
            let mut next = Vec::new();
            let mut queue = World::free_neighbours(&grid, unit.pos)
                .map(|q| (q, 0, q))
                .collect::<VecDeque<_>>();

            while let Some((p, gen, first)) = queue.pop_front() {
                if !encountered.insert(p) {
                    continue;
                }

                if gen != generation {
                    if next.len() > 0 {
                        break;
                    } else {
                        generation = gen;
                    }
                }

                if targets.contains(&p) {
                    next.push((p, first));
                }

                queue.extend(World::free_neighbours(&grid, p).map(|q| (q, gen + 1, first)));
            }

            next.sort_by_key(|&([x1, y1], [x, y])| ([y1, x1], [y, x]));
            next.dedup();

            match next.first().cloned() {
                None => {
                    // eprintln!(
                    //     "unit {}{} cant reach any enemy from {:?}",
                    //     unit.species, unit.id, unit.pos
                    // );
                    continue;
                }
                Some((_, p)) => {
                    // eprintln!("unit {}{} moving to {:?}", unit.species, unit.id, p);
                    let cur = grid
                        .get_mut(&unit.pos)
                        .expect("unit.pos mismatch with grid");
                    *cur = State::Free;
                    unit.pos = p;
                    *grid.get_mut(&p).unwrap() = State::Taken(unit.id);
                    em.update(unit);
                }
            }

            attack(&mut grid, &mut em, unit);
        }

        round += 1;
        World {
            dimensions,
            grid,
            round,
            units,
            em,
        }
    }

    fn outcome(&self) -> u32 {
        self.round * self.remaining_hp()
    }
}

impl str::FromStr for World {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use std::cmp;
        let mut world = World::default();
        let mut top = 0;
        let mut right = 0;
        for (y, l) in s.trim().lines().enumerate() {
            top = cmp::max(top, y);
            for (x, c) in l.trim().chars().enumerate() {
                right = cmp::max(right, x);
                let p = [x, y];
                let state = match c {
                    '#' => State::Wall,
                    '.' => State::Free,
                    'G' | 'E' => {
                        let species = match c {
                            'G' => Species::Goblin,
                            'E' => Species::Elf,
                            _ => unreachable!(),
                        };
                        let id = world.em.create_unit(p, species);
                        world.units.push(id);
                        State::Taken(id)
                    }
                    c => return err!("Failed to parse game world: did not expect character {}", c),
                };

                world.grid.insert(p, state);
            }
        }
        world.dimensions = [right, top];
        Ok(world)
    }
}

impl fmt::Display for World {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "round: {}\nhp remaining: {}",
            self.round,
            self.remaining_hp()
        )?;
        for y in 0..=self.dimensions[1] {
            let mut units = Vec::new();
            for x in 0..=self.dimensions[0] {
                let c = match self.grid[&[x, y]] {
                    State::Free => ".",
                    State::Wall => "#",
                    State::Taken(id) => {
                        let unit = self.em[id].expect("dead units should occur in grid");
                        units.push(unit);
                        write!(f, "{}", unit.species)?;
                        continue;
                    }
                };
                write!(f, "{}", c)?;
            }
            write!(f, "\t")?;
            for u in units {
                write!(f, " {}", u)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
struct EntityManager {
    units: Vec<Option<Unit>>,
    alive: HashMap<Species, usize>,
    elf_power: u8,
}

impl EntityManager {
    fn new(elf_power: u8) -> Self {
        EntityManager {
            elf_power,
            ..EntityManager::default()
        }
    }

    fn create_unit(&mut self, p: Point, species: Species) -> UnitID {
        let id = self.units.len();
        let unit = Unit::new(id, species, p, self.elf_power);
        self.units.push(Some(unit));
        *self.alive.entry(species).or_default() += 1;
        id
    }

    fn get_enemies(&self, s: Species) -> impl Iterator<Item = &Unit> {
        self.units.iter().flatten().filter(move |u| u.species != s)
    }

    fn update(&mut self, unit: Unit) {
        match unit.hp {
            0 => {
                self.units[unit.id] = None;
                let count = self
                    .alive
                    .get_mut(&unit.species)
                    .expect("species should have been registered");
                *count = count.saturating_sub(1);
            }
            _ => self.units[unit.id] = Some(unit),
        }
    }
}

impl ops::Index<UnitID> for EntityManager {
    type Output = Option<Unit>;

    fn index(&self, id: UnitID) -> &Self::Output {
        &self.units[id]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Species {
    Elf,
    Goblin,
}

impl fmt::Display for Species {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Species::Elf => 'E',
            Species::Goblin => 'G',
        };
        write!(f, "{}", c)
    }
}

type UnitID = usize;

#[derive(Clone, Copy, Debug)]
struct Unit {
    id: UnitID,
    hp: u16,
    ap: u8,
    species: Species,
    pos: Point,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.species, self.hp)
    }
}

impl Unit {
    fn new(id: UnitID, species: Species, pos: Point, ap: u8) -> Self {
        Unit {
            id,
            hp: 200,
            ap,
            species,
            pos,
        }
    }
}

fn simulate_battle(mut world: World) -> World {
    while !world.game_over() {
        eprintln!("{}", world);
        world = world.evolve();
    }

    eprintln!("{}", world);
    world
}

fn level1(world: World) -> u32 {
    let world = simulate_battle(world);
    world.outcome()
}

fn level2(world: World) -> u32 {
    let init_elfs = world.em.alive[&Species::Elf];
    let mut i = 4;
    loop {
        let mut world = world.clone();
        world.set_em(EntityManager::new(i));
        let world = simulate_battle(world);
        if world.em.alive[&Species::Elf] == init_elfs {
            return world.outcome();
        } else {
            i += 1;
        }
    }
}

fn solve() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let world = input.parse().unwrap();

    // let some = level1(world);
    // writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(world);
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

    fn check_level1(s: &str, expected: u32, err: &str) {
        let world = s.parse().unwrap();
        let result = level1(world);
        assert_eq!(result, expected, "{}", err);
    }

    fn check_level2(s: &str, expected: u32, err: &str) {
        let world = s.parse().unwrap();
        let result = level2(world);
        assert_eq!(result, expected, "{}", err);
    }

    #[test]
    fn level1_examples() {
        check_level1(E0, 27730, "E0");
        check_level1(E1, 36334, "E1");
        check_level1(E2, 39514, "E2");
        check_level1(E3, 27755, "E3");
        check_level1(E4, 28944, "E4");
        check_level1(E5, 18740, "E5");
        check_level1(REDDIT, 10804, "REDDIT");
    }

    #[test]
    fn level2_examples() {
        check_level2(E1, 4988, "E1");
        check_level2(E2, 31284, "E2");
        check_level2(E3, 3478, "E3");
        check_level2(E4, 6474, "E4");
        check_level2(E5, 1140, "E5");
        check_level2(REDDIT, 517, "REDDIT");
    }

    const E0: &str = "
#######
#.G...#
#...EG#
#.#.#G#
#..G#E#
#.....#
#######";

    const E1: &str = "
#######
#G..#E#
#E#E.E#
#G.##.#
#...#E#
#...E.#
#######";

    const E2: &str = "
#######
#E..EG#
#.#G.E#
#E.##E#
#G..#.#
#..E#.#
#######";

    const E3: &str = "
#######
#E.G#.#
#.#G..#
#G.#.G#
#G..#.#
#...E.#
#######";

    const E4: &str = "
#######
#.E...#
#.#..G#
#.###.#
#E#G#G#
#...#G#
#######";

    const E5: &str = "
#########
#G......#
#.E.#...#
#..##..G#
#...##..#
#...#...#
#.G...G.#
#.....G.#
#########";

    const REDDIT: &str = "
###########
#G..#....G#
###..E#####
###########";
}
