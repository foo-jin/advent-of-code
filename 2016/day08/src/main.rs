#[macro_use]
extern crate nom;

use failure::format_err;
use std::{
    fmt,
    io::{self, BufRead},
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Column,
    Row,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Action {
    Rect {
        x: usize,
        y: usize,
    },
    Shift {
        direction: Direction,
        index: usize,
        amount: usize,
    },
}

impl std::str::FromStr for Action {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use nom::{digit, space, types::CompleteStr as NomInput};

        named!(direction(NomInput) -> Direction,
            alt!(
                value!(Direction::Column, tag!("column")) |
                value!(Direction::Row, tag!("row"))
            )
        );

        named!(rect(NomInput) -> Action,
            do_parse!(
                tag!("rect ") >>
                x: map_res!(terminated!(digit, tag!("x")), |d: NomInput| d.parse()) >>
                y: map_res!(digit, |d: NomInput| d.parse()) >>
                (Action::Rect { x, y } )
            )
        );

        named!(rotate(NomInput) -> Action,
            do_parse!(
                ws!(tag!("rotate")) >>
                direction: direction >>
                space >>
                alt!(tag!("x=") | tag!("y=")) >>
                index: map_res!(terminated!(digit, space), |d:  NomInput| d.parse()) >>
                ws!(tag!("by")) >>
                amount: map_res!(digit, |d: NomInput| d.parse()) >>
                (Action::Shift { direction, index, amount } )
            )
        );

        named!(action(NomInput) -> Action,
            alt!(rect | rotate)
        );

        action(NomInput(s))
            .map(|(_rest, result)| result)
            .map_err(|e| format_err!("Failed to parse input: {}", e))
    }
}

const COLUMNS: usize = 50;
const ROWS: usize = 6;

#[derive(Clone, PartialEq)]
struct Screen {
    state: Vec<Vec<bool>>,
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            state: vec![vec![false; COLUMNS]; ROWS],
        }
    }
}

impl fmt::Debug for Screen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.state {
            for b in row {
                let c = if *b { "#" } else { "." };
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Screen {
    fn execute(&mut self, action: Action) {
        match action {
            Action::Rect { x, y } => {
                self.state[..y]
                    .iter_mut()
                    .flat_map(|xs| xs[..x].iter_mut())
                    .for_each(|x| *x = true);
            }
            Action::Shift {
                direction,
                index,
                amount,
            } => match direction {
                Direction::Column => {
                    let rows = self.state.len();
                    let prev = (0..rows)
                        .map(|r| self.state[r][index])
                        .collect::<Vec<bool>>();
                    for (i, y) in prev.into_iter().enumerate() {
                        self.state[(i + amount) % rows][index] = y;
                    }
                }
                Direction::Row => {
                    self.state[index].rotate_right(amount);
                }
            },
        }
    }

    fn count_on(&self) -> usize {
        self.state
            .iter()
            .flat_map(|row| row.iter())
            .filter(|b| **b)
            .count()
    }
}

fn main() -> Result<(), failure::Error> {
    let mut screen = Screen::default();

    for line in io::stdin().lock().lines() {
        let action = line?.parse()?;
        screen.execute(action);
    }

    eprintln!("{:?}", screen);
    println!("{}", screen.count_on());

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_action(screen: &mut Screen, action: Action, expected: &Screen) {
        screen.execute(action);
        assert_eq!(screen, expected)
    }

    #[test]
    fn actions() {
        let mut result = Screen {
            state: vec![vec![false; 8]; 3],
        };
        let mut expected = result.clone();
        for xs in &mut expected.state[..2] {
            for x in &mut xs[..3] {
                *x = true;
            }
        }

        let action = Action::Rect { x: 3, y: 2 };
        check_action(&mut result, action, &expected);

        expected.state[0][1] = false;
        expected.state[2][1] = true;
        let action = Action::Shift {
            direction: Direction::Column,
            index: 1,
            amount: 1,
        };
        check_action(&mut result, action, &expected);

        expected.state[0][0] = false;
        expected.state[0][2] = false;
        expected.state[0][4] = true;
        expected.state[0][6] = true;
        let action = Action::Shift {
            direction: Direction::Row,
            index: 0,
            amount: 4,
        };
        check_action(&mut result, action, &expected);

        expected.state[0][1] = true;
        expected.state[1][1] = false;
        let action = Action::Shift {
            direction: Direction::Column,
            index: 1,
            amount: 1,
        };
        check_action(&mut result, action, &expected);

        assert_eq!(result.count_on(), 6);
    }

    #[test]
    fn parsing() {
        let s = "rotate column x=1 by 1";
        let result: Action = s.parse().unwrap();
        let expected = Action::Shift {
            direction: Direction::Column,
            index: 1,
            amount: 1,
        };

        assert_eq!(result, expected)
    }
}
