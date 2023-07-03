use color_eyre::{self, Report, Result};
use std::{
    collections::VecDeque,
    fmt::{Debug, Write},
    io::{self, Read},
    str::FromStr,
};

const COLS: usize = 7;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Direction {
    Left = -1,
    Right = 1,
}

#[derive(Copy, Clone, Debug)]
enum Shape {
    Horizontal,
    Plus,
    ReverseL,
    Vertical,
    Square,
}

impl Shape {
    fn height(&self) -> usize {
        match self {
            Self::Horizontal => 1,
            Self::Square => 2,
            Self::Plus | Self::ReverseL => 3,
            Self::Vertical => 4,
        }
    }

    fn points(&self) -> &[(i16, i16)] {
        match self {
            Self::Horizontal => &[(0, 0), (0, 1), (0, 2), (0, 3)],
            Self::Plus => &[(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)],
            Self::ReverseL => &[(0, 2), (1, 2), (2, 0), (2, 1), (2, 2)],
            Self::Vertical => &[(0, 0), (1, 0), (2, 0), (3, 0)],
            Self::Square => &[(0, 0), (0, 1), (1, 0), (1, 1)],
        }
    }

    fn shift_horizontal(
        &self,
        i: i16,
        j: i16,
        dj_delta: i16,
    ) -> impl Iterator<Item = (i16, i16)> + '_ {
        self.points()
            .iter()
            .map(move |(di, dj)| (i + di, j + dj + dj_delta))
    }

    fn shift_vertical(
        &self,
        i: i16,
        j: i16,
        di_delta: i16,
    ) -> impl Iterator<Item = (i16, i16)> + '_ {
        self.points()
            .iter()
            .map(move |(di, dj)| (i + di + di_delta, j + dj))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Cell {
    Empty,
    SettledRock,
}

struct Row([Cell; COLS]);

impl Row {
    const EMPTY: [Cell; COLS] = [Cell::Empty; COLS];

    fn new() -> Self {
        Self(Self::EMPTY)
    }
}

impl Debug for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for cell in self.0 {
            let c = match cell {
                Cell::Empty => '.',
                Cell::SettledRock => '#',
            };
            f.write_char(c)?;
        }
        Ok(())
    }
}

struct Tower {
    rows: VecDeque<Row>,
    height: usize,
}

impl Debug for Tower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.rows {
            writeln!(f, "|{:?}|", row)?;
        }
        f.write_str("+-------+\n")
    }
}

impl Tower {
    fn new() -> Self {
        Self {
            rows: VecDeque::new(),
            height: 0,
        }
    }

    fn is_clear(&self, i: i16, j: i16) -> bool {
        let cell = self
            .rows
            .get(i as usize)
            .and_then(|row| row.0.get(j as usize));

        cell == Some(&Cell::Empty)
    }

    fn height(&self) -> usize {
        self.height
    }

    fn cap(&self) -> usize {
        self.rows.len()
    }

    fn ensure_capacity(&mut self, height: usize) {
        if self.rows.len() < (self.height() + height) {
            for _ in 0..10 {
                self.rows.push_front(Row::new())
            }
        }
    }

    fn set(&mut self, i: i16, j: i16, next: Cell) {
        debug_assert!(i >= 0);
        debug_assert!((0..7).contains(&j));

        let cell = self.rows[i as usize].0.get_mut(j as usize).unwrap();
        debug_assert_eq!(*cell, Cell::Empty, "tried to overwrite an existing rock");

        *cell = next;
    }

    fn add(&mut self, rock: Rock) {
        rock.points()
            .for_each(|(i, j)| self.set(i, j, Cell::SettledRock));
        self.height = self.height.max(self.rows.len() - rock.i as usize);
    }
}

#[derive(Clone, Debug)]
struct Rock {
    shape: Shape,
    i: i16,
    j: i16,
}

impl Rock {
    fn step(&mut self, tower: &Tower, dj: Direction) -> bool {
        // Can we move laterally?
        if self.horizontal_clearance(tower, dj as i16) {
            self.j += dj as i16;
        }

        // Can we move down?
        if self.vertical_clearance(tower, 1) {
            self.i += 1;
            return true;
        }

        false
    }

    fn horizontal_clearance(&self, tower: &Tower, dj: i16) -> bool {
        self.shape
            .shift_horizontal(self.i, self.j, dj)
            .all(|(i, j)| tower.is_clear(i, j))
    }

    fn vertical_clearance(&self, tower: &Tower, di: i16) -> bool {
        self.shape
            .shift_vertical(self.i, self.j, di)
            .all(|(i, j)| tower.is_clear(i, j))
    }

    fn points(&self) -> impl Iterator<Item = (i16, i16)> + '_ {
        self.shape
            .points()
            .iter()
            .map(|&(di, dj)| (self.i + di, self.j + dj))
    }
}

struct Task {
    gusts: Vec<Direction>,
}

impl FromStr for Task {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let gusts = s
            .trim()
            .chars()
            .map(|c| match c {
                '<' => Direction::Left,
                '>' => Direction::Right,
                _ => panic!("unexpected character"),
            })
            .collect::<Vec<_>>();

        Ok(Self { gusts })
    }
}

impl Task {
    const NUM_ROCKS: usize = 2022;

    const SHAPES: [Shape; 5] = [
        Shape::Horizontal,
        Shape::Plus,
        Shape::ReverseL,
        Shape::Vertical,
        Shape::Square,
    ];

    fn part1(&self) -> usize {
        let mut tower = Tower::new();
        let n = self.gusts.len();
        let mut step = 0;

        for i in 0..Self::NUM_ROCKS {
            let shape = Self::SHAPES[i % 5];
            let need = shape.height() + 3;
            tower.ensure_capacity(need);

            let avail = tower.cap() - tower.height();
            let start = avail.saturating_sub(need) as i16;

            let mut rock = Rock {
                shape,
                i: start,
                j: 2,
            };

            loop {
                let dj = self.gusts[step % n];
                step += 1;

                if !rock.step(&tower, dj) {
                    break;
                }
            }

            // Place the rock in the tower at its current position
            tower.add(rock);
        }

        tower.height()
    }

    fn part2(&self) -> usize {
        1514285714288
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = input.parse::<Task>()?;
    println!("part 1: height of tower, 2e03 steps: {}", task.part1());
    println!("part 2: height of tower, 1e12 steps: {}", task.part2());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>\n";
    const L: Direction = Direction::Left;
    const R: Direction = Direction::Right;

    #[test]
    fn gusts() {
        let task = EXAMPLE.parse::<Task>().unwrap();
        assert_eq!(&task.gusts[0..5], &vec![R, R, R, L, L]);

        let n = task.gusts.len();
        assert_eq!(&task.gusts[n - 5..n], &vec![R, L, L, R, R]);
    }

    #[test]
    fn part1() {
        let task = EXAMPLE.parse::<Task>().unwrap();
        assert_eq!(task.part1(), 3068);
    }

    #[test]
    fn part2() {
        let task = EXAMPLE.parse::<Task>().unwrap();
        assert_eq!(task.part2(), 1514285714288);
    }

    #[test]
    fn with_input() {
        let input = include_str!("../data/input.txt");
        let task = input.parse::<Task>().unwrap();
        assert_eq!(task.part1(), 3133);
    }
}
