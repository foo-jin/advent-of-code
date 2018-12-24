use std::{
    cmp,
    collections::{HashMap, HashSet},
    error::Error,
    io::{self, Read, Write},
    str::FromStr,
};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<std::error::Error>::from(format!($($tt)*))) }
}

macro_rules! format_err {
    ($($tt:tt)*) => { Box::<std::error::Error>::from(format!($($tt)*)) }
}

mod aoc {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}

fn solve() -> aoc::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let some = level1(&input);
    writeln!(io::stderr(), "level 1: {}", some)?;

    let thing = level2(&input);
    writeln!(io::stderr(), "level 2: {}", thing)?;

    // stdout is used to submit solutions
    writeln!(io::stdout(), "{}", thing)?;
    Ok(())
}

fn level1(s: &str) -> u32 {
    let mut world = s.parse::<World>().unwrap();
    world.simulate_battle().1
}

fn level2(s: &str) -> u32 {
    let world = s.parse::<World>().unwrap();
    let mut boost = 0;
    loop {
        let mut world = world.clone();
        world.boost = boost;
        if let (Army::ImmuneSys, score) = world.simulate_battle() {
            return score;
        }
        boost += 1;
    }
}

/// should be faster when the first useful boost is a high number
/// unfortunately this is not the case for my input
fn level2_bin_search(s: &str) -> u32 {
    use self::Army::*;

    let world = s.parse::<World>().unwrap();
    let mut prev = 0;
    let mut boost = 1;
    loop {
        let mut w1 = world.clone();
        let mut w2 = world.clone();
        w1.boost = boost - 1;
        w2.boost = boost;

        let (a1, _) = w1.simulate_battle();
        let (a2, score) = w2.simulate_battle();
        match (a1, a2) {
            (Infection, ImmuneSys) => return score,
            (ImmuneSys, ImmuneSys) => boost = prev + ((boost - prev) / 2),
            (Infection, Infection) => {
                prev = boost;
                boost *= 2;
            }
            _ => panic!("a higher boost can never lead to immune system losing"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Army {
    Infection,
    ImmuneSys,
}

type Idx = (Army, usize);

#[derive(Clone, Debug)]
struct World {
    boost: u32,
    atk_order: Vec<Idx>,
    immuno: Vec<Group>,
    infection: Vec<Group>,
}

impl World {
    fn game_over(&self) -> Option<(Army, u32)> {
        let immuno = self.immuno.iter().map(|g| g.units).sum();
        let infection = self.infection.iter().map(|g| g.units).sum();
        match (immuno, infection) {
            (0, score) => Some((Army::Infection, score)),
            (score, 0) => Some((Army::ImmuneSys, score)),
            _ => None,
        }
    }

    fn simulate_battle(&mut self) -> (Army, u32) {
        let boost = self.boost;
        self.immuno.iter_mut().for_each(|g| g.ad += boost);
        loop {
            if let Some(result) = self.game_over() {
                return result;
            }

            self.clean_groups();
            self.log_state();
            self.select_targets(Army::Infection);
            self.select_targets(Army::ImmuneSys);

            log::debug!("");
            let mut killed = false;
            for i in 0..self.atk_order.len() {
                let id = self.atk_order[i];
                let (allies, enemies) = self.relative_groups(id.0);
                let group = allies.iter().find(|g| g.id == id).unwrap();
                if !group.alive() {
                    continue;
                }

                if let Some(target) = group.target {
                    let mut target = enemies.iter_mut().find(|g| g.id == target).unwrap();
                    killed |= group.attack(&mut target);
                }
            }

            if !killed {
                return (Army::Infection, 0);
            }
        }
    }

    fn relative_groups(&mut self, a: Army) -> (&mut [Group], &mut [Group]) {
        match a {
            Army::Infection => (&mut self.infection, &mut self.immuno),
            Army::ImmuneSys => (&mut self.immuno, &mut self.infection),
        }
    }

    fn select_targets(&mut self, a: Army) {
        let (allies, enemies) = self.relative_groups(a);
        let mut taken = HashSet::new();
        for atk in allies {
            let target = enemies
                .iter()
                .filter(|def| !taken.contains(&def.id))
                .map(|def| (atk.calc_dmg(def), def))
                .inspect(|(dmg, def)| {
                    let (a, i) = atk.id;
                    let (_, j) = def.id;
                    log::debug!(
                        "{:?} group {} would deal defending group {} {} damage",
                        a,
                        i,
                        j,
                        dmg,
                    )
                })
                .max_by_key(|&(dmg, def)| (dmg, def.ep(), def.initiative));

            match target {
                Some((0, _)) | None => atk.target = None,
                Some((_, def)) => {
                    atk.target = Some(def.id);
                    taken.insert(def.id);
                }
            }
        }
    }

    fn clean_groups(&mut self) {
        self.immuno.retain(|g| g.alive());
        self.infection.retain(|g| g.alive());
        let either = self
            .immuno
            .iter()
            .chain(self.infection.iter())
            .map(|g| g.id)
            .collect::<HashSet<Idx>>();
        self.atk_order.retain(|id| either.contains(id));

        let key = |g: &Group| cmp::Reverse((g.ep(), g.initiative));
        self.immuno.sort_by_key(key);
        self.infection.sort_by_key(key);
    }

    fn log_state(&self) {
        let mut msg = "\nImmune system:\n".to_owned();
        for g in &self.immuno {
            msg.push_str(&format!("Group {} contains {} units\n", g.id.1, g.units));
        }
        msg.push_str(&format!(
            "total: {}\n",
            self.immuno.iter().map(|g| g.units).sum::<u32>()
        ));
        msg.push_str("Infection:\n");
        for g in &self.infection {
            msg.push_str(&format!("Group {} contains {} units\n", g.id.1, g.units));
        }
        msg.push_str(&format!(
            "total: {}\n",
            self.infection.iter().map(|g| g.units).sum::<u32>()
        ));
        log::debug!("{}", msg);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum DamageKind {
    Fire,
    Cold,
    Radiation,
    Bludgeoning,
    Slashing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Affinity {
    Weak,
    Immune,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Group {
    id: Idx,
    units: u32,
    hp: u32,
    ad: u32,
    atype: DamageKind,
    initiative: u8,
    matchups: HashMap<DamageKind, Affinity>,
    target: Option<Idx>,
}

impl Group {
    fn alive(&self) -> bool {
        self.units > 0
    }

    fn ep(&self) -> u32 {
        self.units * self.ad
    }

    fn calc_dmg(&self, other: &Group) -> u32 {
        let dmg = self.ep();
        match other.matchups.get(&self.atype) {
            None => dmg,
            Some(Affinity::Weak) => dmg * 2,
            Some(Affinity::Immune) => 0,
        }
    }

    fn attack(&self, other: &mut Group) -> bool {
        let killed = other.damage(self.calc_dmg(other));
        log::debug!(
            "{:?} group {} attacks defending group {}, killing {} units",
            self.id.0,
            self.id.1,
            other.id.1,
            killed,
        );
        killed > 0
    }

    fn damage(&mut self, amount: u32) -> u32 {
        let killed = self.units.min(amount / self.hp);
        self.units -= killed;
        killed
    }
}

impl FromStr for DamageKind {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::DamageKind::*;
        let kind = match s {
            "fire" => Fire,
            "cold" => Cold,
            "radiation" => Radiation,
            "bludgeoning" => Bludgeoning,
            "slashing" => Slashing,
            _ => return err!("unkown damage type: {}", s),
        };
        Ok(kind)
    }
}

impl FromStr for Group {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use regex::Regex;
        lazy_static::lazy_static! {
            static ref RE: Regex = Regex::new(r"(?x)
                (?P<units>[0-9]+)(?-x: units each with )
                (?P<hp>[0-9]+)(?-x: hit points)
                (?P<inner>\s\([^\)]*\))?(?-x: with an attack that does )
                (?P<ad>[0-9]+)\s
                (?P<atype>[a-z]+)(?-x: damage at initiative )
                (?P<init>[0-9]+)
            ").unwrap();
        }
        let caps = RE
            .captures(s.trim())
            .ok_or_else(|| format_err!("input not captured by regex: {}", s))?;
        log::trace!("{:?}", caps);
        let units = caps["units"].parse()?;
        let hp = caps["hp"].parse()?;
        let ad = caps["ad"].parse()?;
        let atype = caps["atype"].parse()?;
        let initiative = caps["init"].parse()?;

        let mut matchups = HashMap::new();
        if let Some(inner) = caps.name("inner") {
            let s = inner
                .as_str()
                .trim_matches(|c: char| c == '(' || c == ')' || c.is_whitespace());
            log::trace!("inner: {}", s);
            if s.contains(';') {
                let parts = s.split(';').collect::<Vec<_>>();
                parse_matchups(parts[0].trim(), &mut matchups)?;
                parse_matchups(parts[1].trim(), &mut matchups)?;
            } else {
                parse_matchups(s, &mut matchups)?;
            }
        }
        log::trace!("matchups: {:?}", matchups);

        let result = Group {
            id: (Army::Infection, 0),
            units,
            hp,
            ad,
            atype,
            initiative,
            matchups,
            target: None,
        };
        Ok(result)
    }
}

impl FromStr for World {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split("\n\n").collect();
        let immuno = {
            let s = parts[0];
            assert!(s.starts_with("Immune System:"));
            parse_groups(s, Army::ImmuneSys)?
        };

        let infection = {
            let s = parts[1];
            assert!(s.starts_with("Infection:"));
            parse_groups(s, Army::Infection)?
        };

        let mut atk_order: Vec<Idx> = immuno
            .iter()
            .chain(infection.iter())
            .map(|g| g.id)
            .collect();

        atk_order.sort_by_key(|(army, i)| match army {
            Army::ImmuneSys => cmp::Reverse(immuno[i - 1].initiative),
            Army::Infection => cmp::Reverse(infection[i - 1].initiative),
        });

        Ok(World {
            boost: 0,
            atk_order,
            immuno,
            infection,
        })
    }
}

fn parse_matchups(s: &str, matchups: &mut HashMap<DamageKind, Affinity>) -> aoc::Result<()> {
    let parts = s.split(" to ").collect::<Vec<_>>();
    let dmgkinds = parse_dmgkind_list(parts[1])?;
    let aff = match parts[0] {
        "weak" => Affinity::Weak,
        "immune" => Affinity::Immune,
        s => return err!("unkown affinity: {}", s),
    };
    matchups.extend(dmgkinds.into_iter().map(|d| (d, aff)));
    Ok(())
}

fn parse_dmgkind_list(s: &str) -> aoc::Result<Vec<DamageKind>> {
    s.split(", ").map(str::parse).collect()
}

fn parse_groups(s: &str, army: Army) -> aoc::Result<Vec<Group>> {
    s.lines()
        .enumerate()
        .skip(1)
        .map(|(i, l)| {
            l.parse::<Group>().map(|mut g| {
                g.id = (army, i);
                g
            })
        })
        .collect()
}

fn main() -> aoc::Result<()> {
    env_logger::init();
    if let Err(e) = solve() {
        let stderr = io::stderr();
        let mut w = stderr.lock();
        writeln!(w, "Error: {}", e)?;
        while let Some(e) = e.source() {
            writeln!(w, "\t{}", e)?;
        }

        std::process::exit(-1)
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    const INPUT: &str = include_str!("../input.txt");

    const EX: &str = "
Immune System:
17 units each with 5390 hit points (weak to radiation, bludgeoning) with an attack that does 4507 fire damage at initiative 2
989 units each with 1274 hit points (immune to fire; weak to bludgeoning, slashing) with an attack that does 25 slashing damage at initiative 3

Infection:
801 units each with 4706 hit points (weak to radiation) with an attack that does 116 bludgeoning damage at initiative 1
4485 units each with 2961 hit points (immune to radiation; weak to fire, cold) with an attack that does 12 slashing damage at initiative 4";

    #[test_log::new]
    fn level1_examples() {
        assert_eq!(level1(EX), 5216)
    }

    #[test_log::new]
    fn level2_examples() {
        assert_eq!(level2(EX), 51)
    }
}
