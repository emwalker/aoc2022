use color_eyre::{self, Report, Result};
use std::{
    fmt::{Debug, Write},
    str::FromStr,
};

type Int = i16;
type Point = (Int, Int);

const COLS: usize = 7;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Direction {
    Left = -1,
    Right = 1,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Shape {
    Horizontal,
    Plus,
    ReverseL,
    Vertical,
    Square,
}

impl Shape {
    fn points(&self) -> &[Point] {
        match self {
            Self::Horizontal => &[(0, 0), (0, 1), (0, 2), (0, 3)],
            Self::Plus => &[(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)],
            Self::ReverseL => &[(0, 0), (0, 1), (0, 2), (1, 2), (2, 2)],
            Self::Vertical => &[(0, 0), (1, 0), (2, 0), (3, 0)],
            Self::Square => &[(0, 0), (0, 1), (1, 0), (1, 1)],
        }
    }

    fn shift_horizontal(&self, p: Point, dj_delta: Int) -> impl Iterator<Item = Point> + '_ {
        self.points()
            .iter()
            .map(move |(di, dj)| (p.0 + di, p.1 + dj + dj_delta))
    }

    fn shift_vertical(&self, p: Point, di_delta: Int) -> impl Iterator<Item = Point> + '_ {
        self.points()
            .iter()
            .map(move |(di, dj)| (p.0 + di + di_delta, p.1 + dj))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Cell {
    Empty,
    SettledRock,
}

pub struct Row([Cell; COLS]);

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

struct Chamber {
    pub rows: Vec<Row>,
    pub max_i_by_col: [Int; COLS],
    pub max_i: Int,
}

impl Debug for Chamber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("\n|-------|\n")?;
        for row in self.rows.iter().rev() {
            writeln!(f, "|{:?}|", row)?;
        }
        f.write_str("+-------+\n")
    }
}

impl Chamber {
    fn new() -> Self {
        Self {
            rows: Vec::with_capacity(4096),
            max_i_by_col: [-1; COLS],
            max_i: -1,
        }
    }

    fn is_available(&self, p: Point) -> bool {
        let (i, j) = p;

        if j < 0 {
            return false;
        }
        let j = j as usize;

        if j >= COLS {
            return false;
        }

        if i < 0 {
            return false;
        }
        let i = i as usize;

        // If i goes beyond the current capacity of the chamber, there are no obstructions, and the
        // block can be placed here, assuming additional capacity is added.
        if i >= self.rows.len() {
            return true;
        }

        self.rows[i].0.get(j) == Some(&Cell::Empty)
    }

    pub fn height(&self) -> Int {
        self.max_i + 1
    }

    fn set(&mut self, p: Point, next: Cell) {
        debug_assert!(p.0 >= 0);
        let i = p.0 as usize;

        debug_assert!((0..7).contains(&p.1));

        if i >= self.rows.len() {
            for _ in 0..10 {
                self.rows.push(Row::new());
            }
        }

        let cell = self.rows[i]
            .0
            .get_mut(p.1 as usize)
            .expect("p.1 within column bounds");

        debug_assert_eq!(*cell, Cell::Empty, "tried to overwrite an existing rock");

        *cell = next;
    }

    fn insert(&mut self, rock: Rock) {
        let mut max_i = -1;

        for p in rock.points() {
            let j = p.1 as usize;
            self.max_i_by_col[j] = self.max_i_by_col[j].max(p.0);
            max_i = max_i.max(self.max_i_by_col[j]);
            self.set(p, Cell::SettledRock);
        }

        self.max_i = self.max_i.max(max_i);
    }
}

#[derive(Clone, Debug)]
struct Rock {
    shape: Shape,
    bottom_left: Point,
}

impl Rock {
    fn step(&mut self, chamber: &Chamber, dj: Direction) -> bool {
        // Can we move laterally?
        if self.horizontal_clearance(chamber, dj as Int) {
            self.bottom_left.1 += dj as Int;
        }

        // Can we move down?
        if self.vertical_clearance(chamber, -1) {
            self.bottom_left.0 -= 1;
            return true;
        }

        false
    }

    fn horizontal_clearance(&self, chamber: &Chamber, dj: Int) -> bool {
        self.shape
            .shift_horizontal(self.bottom_left, dj)
            .all(|p| chamber.is_available(p))
    }

    fn vertical_clearance(&self, chamber: &Chamber, di: Int) -> bool {
        self.shape
            .shift_vertical(self.bottom_left, di)
            .all(|p| chamber.is_available(p))
    }

    fn points(&self) -> impl Iterator<Item = Point> + '_ {
        let p = self.bottom_left;
        self.shape
            .points()
            .iter()
            .map(move |&(di, dj)| (p.0 + di, p.1 + dj))
    }
}

pub struct Task {
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
    const NUM_SHAPES: usize = 5;

    const SHAPES: [Shape; Self::NUM_SHAPES] = [
        Shape::Horizontal,
        Shape::Plus,
        Shape::ReverseL,
        Shape::Vertical,
        Shape::Square,
    ];

    pub fn height_of_tower(&self, num_rocks: usize) -> Int {
        let (chamber, _rock, _step) = self.state_at(num_rocks);
        chamber.height()
    }

    fn state_at(&self, num_rocks: usize) -> (Chamber, Option<Rock>, usize) {
        let mut chamber = Chamber::new();
        let n = self.gusts.len();
        let mut step = 0;
        let mut last_rock = None;

        for r in 0..num_rocks {
            let mut rock = Rock {
                shape: Self::SHAPES[r % Self::NUM_SHAPES],
                bottom_left: (chamber.max_i + 4, 2),
            };

            while rock.step(&chamber, self.gusts[step % n]) {
                step += 1;
            }
            step += 1;

            last_rock = Some(rock.clone());
            chamber.insert(rock);
        }

        (chamber, last_rock, step)
    }
}

pub fn parse(input: &str) -> Result<Task> {
    input.parse::<Task>()
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
        assert_eq!(task.height_of_tower(2022), 3068);
    }

    #[test]
    fn with_input() {
        let input = include_str!("../data/input.txt");
        let task = input.parse::<Task>().unwrap();
        assert_eq!(task.height_of_tower(2022), 3133);
    }

    #[test]
    fn subtle_bug() {
        let input = include_str!("../data/input.txt");
        let task = input.parse::<Task>().unwrap();
        let (chamber, rock, step) = task.state_at(25);

        assert_eq!(step, 145);
        assert_eq!(chamber.height(), 39);
        assert_eq!(rock.unwrap().shape, Shape::Square);

        let (chamber, rock, step) = task.state_at(26);
        assert_eq!(step, 149);
        assert_eq!(chamber.height(), 40);
        assert_eq!(rock.unwrap().shape, Shape::Horizontal);

        let (chamber, rock, step) = task.state_at(27);
        assert_eq!(step, 165);
        assert_eq!(chamber.height(), 40);
        assert_eq!(rock.unwrap().shape, Shape::Plus);
    }
}
