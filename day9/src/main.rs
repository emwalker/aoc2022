use color_eyre::{self, eyre::eyre, Report, Result};
use itertools::Itertools;
use num::{pow, Complex};
use std::{
    collections::HashSet,
    f32::MAX,
    fmt::Debug,
    io::{self, Read},
    str::FromStr,
};

type Position = Complex<i32>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn to_complex(self) -> Position {
        match self {
            Self::Up => Complex::new(1, 0),
            Self::Right => Complex::new(0, 1),
            Self::Down => Complex::new(-1, 0),
            Self::Left => Complex::new(0, -1),
        }
    }
}

#[derive(Clone, Debug)]
struct Instruction {
    steps: isize,
    dir: Direction,
}

impl FromStr for Instruction {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (dir, steps) = s
            .split(' ')
            .collect_tuple()
            .ok_or(eyre!("bad input: {s}"))?;
        let steps = steps.parse::<isize>()?;

        let direction = match dir {
            "U" => Direction::Up,
            "R" => Direction::Right,
            "D" => Direction::Down,
            "L" => Direction::Left,
            _ => return Err(eyre!("bad direction: {dir}")),
        };

        Ok(Self {
            dir: direction,
            steps,
        })
    }
}

impl Instruction {
    fn decrement(&mut self) {
        self.steps -= 1;
    }

    fn is_empty(&self) -> bool {
        self.steps <= 0
    }
}

#[derive(Clone, Debug)]
struct Instructions(Vec<Instruction>);

impl Instructions {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn pop(&mut self) -> Option<Instruction> {
        self.0.pop()
    }

    fn last_mut(&mut self) -> Option<&mut Instruction> {
        self.0.last_mut()
    }
}

struct Knot {
    pos: Position,
}

impl Debug for Knot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({},{})", self.pos.re, self.pos.im))
    }
}

impl Knot {
    fn new(pos: Position) -> Self {
        Self { pos }
    }

    fn apply(&mut self, instructions: &mut Instructions) {
        if let Some(ins) = instructions.last_mut() {
            let step = ins.dir.to_complex();
            self.pos += step;
            ins.decrement();

            if ins.is_empty() {
                instructions.pop();
            }
        }
    }

    fn neighbors(&self) -> Vec<Position> {
        [-1, 0, 1]
            .iter()
            .flat_map(|&re| {
                [-1, 0, 1]
                    .iter()
                    .map(move |&im| self.pos + Complex::new(re, im))
            })
            .collect_vec()
    }

    fn four_ways(&self) -> Vec<Position> {
        [(1, 0), (0, 1), (-1, 0), (0, -1)]
            .into_iter()
            .map(|(re, im)| self.pos + Complex::new(re, im))
            .collect_vec()
    }
}

struct Task {
    ins: Instructions,
}

impl Task {
    fn parse(lines: &[String]) -> Result<Self> {
        let ins = lines
            .iter()
            .rev()
            .map(|l| l.trim().parse::<Instruction>())
            .collect::<Result<Vec<Instruction>>>()?;

        Ok(Self {
            ins: Instructions(ins),
        })
    }

    fn positions_visited_by_tail(&self) -> usize {
        let mut instructions = self.ins.clone();
        let mut visited = HashSet::new();
        let pos = Complex::new(0, 0);
        visited.insert(pos);

        let (mut head, mut tail) = (Knot::new(pos), Knot::new(pos));

        fn distance(p1: Position, p2: Position) -> f32 {
            (pow(p1.im as f32 - p2.im as f32, 2) + pow(p1.re as f32 - p2.re as f32, 2)).sqrt()
        }

        while !instructions.is_empty() {
            head.apply(&mut instructions);

            if head.neighbors().contains(&tail.pos) {
                continue;
            }

            let mut dmin = MAX;
            let mut next = tail.pos;

            for near in head.four_ways() {
                let d = distance(tail.pos, near);
                if d < dmin {
                    next = near;
                    dmin = d;
                }
            }
            tail.pos = next;
            visited.insert(next);
        }

        visited.len()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let lines = input.lines().map(str::to_owned).collect_vec();

    let task = Task::parse(&lines)?;
    println!("positions visited: {}", task.positions_visited_by_tail());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> &'static str {
        "R 4
         U 4
         L 3
         D 1
         R 4
         D 1
         L 5
         R 2"
    }

    fn task() -> Task {
        let lines = input().lines().map(str::to_string).collect_vec();
        Task::parse(&lines).unwrap()
    }

    #[test]
    fn part1() {
        let task = task();
        assert!(!task.ins.is_empty());
        assert_eq!(task.positions_visited_by_tail(), 13);
    }
}
