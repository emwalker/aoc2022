use color_eyre::{self, eyre::eyre, Report, Result};
use itertools::Itertools;
use num::{pow, Complex};
use std::{
    collections::HashSet,
    fmt::Debug,
    i32::MAX,
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
    fn pop(&mut self) -> Option<Instruction> {
        self.0.pop()
    }

    fn last_mut(&mut self) -> Option<&mut Instruction> {
        self.0.last_mut()
    }
}

impl Iterator for Instructions {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ins) = self.last_mut() {
            let step = ins.dir.to_complex();
            ins.decrement();

            if ins.is_empty() {
                self.pop();
            }

            return Some(step);
        }

        None
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct Knot(Position);

impl Debug for Knot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({},{})", self.0.re, self.0.im))
    }
}

impl Default for Knot {
    fn default() -> Self {
        Knot(Complex::new(0, 0))
    }
}

impl Knot {
    #[allow(unused)]
    fn new(i: i32, j: i32) -> Self {
        Self(Complex::new(i, j))
    }

    fn step(&mut self, step: Position) -> Self {
        Self(self.0 + step)
    }

    fn follow(&self, prev_knot: Self) -> Option<Self> {
        fn distance(p1: Position, p2: Position) -> i32 {
            pow(p1.re - p2.re, 2) + pow(p1.im - p2.im, 2)
        }

        if prev_knot.neighbors().contains(&self.0) {
            return None;
        }

        let possible_moves = if self.in_line_with(prev_knot) {
            self.four_ways()
        } else {
            self.diagonals()
        };

        let mut dmin = MAX;
        let mut next = self.0;

        for pos in possible_moves {
            let d = distance(prev_knot.0, pos);
            if d < dmin {
                next = pos;
                dmin = d;
            }
        }

        Some(Knot(next))
    }

    fn neighbors(&self) -> Vec<Position> {
        [-1, 0, 1]
            .iter()
            .flat_map(|&re| {
                [-1, 0, 1]
                    .iter()
                    .map(move |&im| self.0 + Complex::new(re, im))
            })
            .collect_vec()
    }

    fn four_ways(&self) -> Vec<Position> {
        [(1, 0), (0, 1), (-1, 0), (0, -1)]
            .into_iter()
            .map(|(re, im)| self.0 + Complex::new(re, im))
            .collect_vec()
    }

    fn diagonals(&self) -> Vec<Position> {
        [(1, 1), (-1, 1), (-1, -1), (1, -1)]
            .into_iter()
            .map(|(re, im)| self.0 + Complex::new(re, im))
            .collect_vec()
    }

    fn in_line_with(&self, other: Knot) -> bool {
        self.0.re == other.0.re || self.0.im == other.0.im
    }
}

#[derive(Clone, Debug)]
struct Rope(Vec<Knot>);

impl Rope {
    fn with_capacity(n: usize) -> Result<Self> {
        if n < 2 {
            return Err(eyre!("capacity cannot be less than 2"));
        }

        let v: Vec<Knot> = vec![Knot::default(); n];
        Ok(Self(v))
    }

    fn step(&self, step: Position) -> Self {
        let mut next_rope = self.clone();
        let mut prev_knot = next_rope.0[0].step(step);
        next_rope.0[0] = prev_knot;

        for (i, u) in self.0.iter().enumerate() {
            if i == 0 {
                continue;
            }

            if let Some(v) = u.follow(prev_knot) {
                next_rope.0[i] = v;
                prev_knot = v;
                continue;
            }

            break;
        }

        next_rope
    }

    fn tail(&self) -> Knot {
        if let Some(knot) = self.0.last() {
            return *knot;
        }
        unreachable!("rope has at least two knots");
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

    fn part1(&self) -> usize {
        self.positions_visited_by_tail(2).unwrap()
    }

    fn part2(&self) -> usize {
        self.positions_visited_by_tail(10).unwrap()
    }

    fn run_scenario(&self, n: usize) -> Result<(Rope, HashSet<Knot>)> {
        let mut prev = Rope::with_capacity(n)?;
        let mut visited = HashSet::from([prev.tail()]);

        for i in self.ins.clone() {
            let next = prev.step(i);
            visited.insert(next.tail());
            prev = next;
        }

        Ok((prev, visited))
    }

    fn positions_visited_by_tail(&self, n: usize) -> Result<usize> {
        let (_rope, visited) = self.run_scenario(n)?;
        Ok(visited.len())
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let lines = input.lines().map(str::to_owned).collect_vec();

    let task = Task::parse(&lines)?;
    println!("positions visited, n=2:  {}", task.part1());
    println!("positions visited, n=10: {}", task.part2());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(input: &str) -> Task {
        let lines = input.lines().map(str::to_string).collect_vec();
        Task::parse(&lines).unwrap()
    }

    #[test]
    fn part1() {
        let input = "\
        R 4
        U 4
        L 3
        D 1
        R 4
        D 1
        L 5
        R 2";

        let task = task(input);
        assert!(!task.ins.0.is_empty());
        assert_eq!(task.part1(), 13);
    }

    #[test]
    fn diagonal_move() {
        let input = "\
        R 4
        U 4";

        let task = task(input);
        let (rope, _visited) = task.run_scenario(10).unwrap();
        assert_eq!(
            rope.0,
            vec![
                Knot::new(4, 4),
                Knot::new(3, 4),
                Knot::new(2, 4),
                Knot::new(2, 3),
                Knot::new(2, 2),
                Knot::new(1, 1),
                Knot::default(),
                Knot::default(),
                Knot::default(),
                Knot::default(),
            ]
        );
    }

    #[test]
    fn part2() {
        let input = "\
        R 5
        U 8
        L 8
        D 3
        R 17
        D 10
        L 25
        U 20";

        let task = task(input);
        assert!(!task.ins.0.is_empty());
        assert_eq!(task.part2(), 36);
    }
}
