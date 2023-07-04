use ahash::RandomState;
use color_eyre::{self, Report, Result};
use std::{
    collections::{BTreeMap, HashSet},
    fmt::{Debug, Write},
    str::FromStr,
};

type Int = i16;
type Point = (Int, Int);

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
    fn points(&self) -> &[Point] {
        match self {
            Self::Horizontal => &[(0, 0), (0, 1), (0, 2), (0, 3)],
            Self::Plus => &[(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)],
            Self::ReverseL => &[(0, 0), (0, 1), (0, 2), (1, 2), (2, 2)],
            Self::Vertical => &[(0, 0), (1, 0), (2, 0), (3, 0)],
            Self::Square => &[(0, 0), (0, 1), (1, 0), (1, 1)],
        }
    }

    fn shift_horizontal(&self, (i, j): Point, dj_delta: Int) -> impl Iterator<Item = Point> + '_ {
        self.points()
            .iter()
            .map(move |(di, dj)| (i + di, j + dj + dj_delta))
    }

    fn shift_vertical(&self, (i, j): Point, di_delta: Int) -> impl Iterator<Item = Point> + '_ {
        self.points()
            .iter()
            .map(move |(di, dj)| (i + di + di_delta, j + dj))
    }
}

const COLS: usize = 7;

struct Chamber {
    points: HashSet<Point, RandomState>,
    max_i: [Int; COLS],
    height: Int,
}

impl Debug for Chamber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rows = BTreeMap::<Int, [Point; COLS]>::new();

        for point in &self.points {
            let row = rows.entry(-point.0).or_insert([(-1, -1); COLS]);
            row[point.1 as usize] = *point;
        }

        f.write_str("\n\n|.......|\n")?;

        for (_i, row) in rows.iter() {
            f.write_char('|')?;
            for p in row {
                let c = if p.0 > -1 { '#' } else { '.' };
                f.write_char(c)?;
            }
            f.write_str("|\n")?;
        }

        f.write_str("+-------+\n")
    }
}

impl Chamber {
    fn new() -> Self {
        Self {
            points: HashSet::<Point, RandomState>::default(),
            max_i: [0; COLS],
            height: 0,
        }
    }

    fn is_available(&self, p: &Point) -> bool {
        (p.1 >= 0 && (p.1 as usize) < COLS) && p.0 > 0 && !self.points.contains(p)
    }

    fn insert(&mut self, rock: Rock) {
        let points = rock.points().collect::<Vec<_>>();
        let mut height = 0;

        for &(i, j) in &points {
            let j = j as usize;
            self.max_i[j] = self.max_i[j].max(i);
            height = height.max(self.max_i[j]);
        }

        self.points.extend(points);
        self.height = self.height.max(height);
    }

    fn height(&self) -> Int {
        self.height
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
            .all(|point| chamber.is_available(&point))
    }

    fn vertical_clearance(&self, chamber: &Chamber, di: Int) -> bool {
        self.shape
            .shift_vertical(self.bottom_left, di)
            .all(|point| chamber.is_available(&point))
    }

    fn points(&self) -> impl Iterator<Item = Point> + '_ {
        let (i, j) = self.bottom_left;
        self.shape
            .points()
            .iter()
            .map(move |&(di, dj)| (i + di, j + dj))
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
        let mut chamber = Chamber::new();
        let n = self.gusts.len();
        let mut step = 0;

        for i in 0..num_rocks {
            let mut rock = Rock {
                shape: Self::SHAPES[i % Self::NUM_SHAPES],
                bottom_left: (chamber.height() + 4, 2),
            };

            loop {
                let dj = self.gusts[step % n];
                step += 1;
                if !rock.step(&chamber, dj) {
                    break;
                }
            }

            // Place the rock in the tower at its current position
            chamber.insert(rock);
        }

        chamber.height()
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
}
