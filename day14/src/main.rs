// Re-worked along the lines of Amos in https://fasterthanli.me/series/advent-of-code-2022/part-14
#![feature(iter_from_generator)]
#![feature(generators)]
#![feature(drain_filter)]

use color_eyre::{self, eyre::eyre, Report, Result};
use derive_more::{Add, AddAssign, Sub};
use itertools::Itertools;
use std::{
    fmt::Debug,
    io::{self, Read},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Add, AddAssign, Sub)]
struct Point {
    x: i32,
    y: i32,
}

const SPAWN_POINT: Point = Point { x: 500, y: 0 };

impl FromStr for Point {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (x, y) = s
            .split(',')
            .collect_tuple()
            .ok_or(eyre!("bad input: {s}"))?;

        Ok(Self {
            x: x.parse()?,
            y: y.parse()?,
        })
    }
}

impl Point {
    fn signum(&self) -> Self {
        Self {
            x: self.x.signum(),
            y: self.y.signum(),
        }
    }
}

#[derive(Clone, Debug)]
struct Polyline {
    points: Vec<Point>,
}

impl FromStr for Polyline {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let points = s
            .trim()
            .split(" -> ")
            .map(Point::from_str)
            .collect::<Result<Vec<Point>>>()?;

        Ok(Self { points })
    }
}

impl Polyline {
    fn path_points(&self) -> impl Iterator<Item = Point> + '_ {
        std::iter::from_generator(|| {
            let mut points = self.points.iter().copied();
            let Some(mut a) = points.next() else { return };
            yield a;

            loop {
                let Some(b) = points.next() else { return };
                let delta = (b - a).signum();

                loop {
                    a += delta;
                    yield a;
                    if a == b {
                        break;
                    }
                }
            }
        })
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Cell {
    Air,
    Rock,
    Sand,
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Air => ".",
            Self::Rock => "#",
            Self::Sand => "o",
        };
        write!(f, "{s}")
    }
}

struct Polylines(Vec<Polyline>);

impl FromStr for Polylines {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(
            s.lines()
                .map(Polyline::from_str)
                .collect::<Result<Vec<Polyline>>>()?,
        ))
    }
}

impl Polylines {
    fn with(&self, polyline: Polyline) -> Self {
        let mut inner = self.0.clone();
        inner.push(polyline);
        Self(inner)
    }

    fn points(&self) -> impl Iterator<Item = Point> + '_ {
        self.0
            .iter()
            .flat_map(|polyline| polyline.path_points())
            .chain(std::iter::once(SPAWN_POINT))
    }

    fn dimensions(&self) -> (i32, i32, i32, i32) {
        let (mut xmin, mut xmax, mut ymin, mut ymax) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);

        for p in self.points() {
            xmin = xmin.min(p.x);
            xmax = xmax.max(p.x);
            ymin = ymin.min(p.y);
            ymax = ymax.max(p.y);
        }

        (xmin, xmax, ymin, ymax)
    }

    fn to_grid(&self) -> Result<Grid> {
        let (xmin, xmax, ymin, ymax) = self.dimensions();
        let origin = Point { x: xmin, y: ymin };
        let height = (ymax - ymin + 1).try_into()?;
        let width = (xmax - xmin + 1).try_into()?;
        let cells = vec![Cell::Air; height * width];

        let mut grid = Grid {
            origin,
            height,
            width,
            cells,
        };

        for p in self.points() {
            *grid.cell_mut(p).unwrap() = Cell::Rock;
        }

        Ok(grid)
    }
}

#[derive(Clone)]
struct Grid {
    origin: Point,
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl FromStr for Grid {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        s.parse::<Polylines>()?.to_grid()
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let p = Point {
                    x: x as _,
                    y: y as _,
                } + self.origin;
                let cell = self.cell(p).unwrap();
                write!(f, "{cell:?}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Grid {
    fn index_of(&self, p: Point) -> Option<usize> {
        let Point { x, y } = p - self.origin;
        let x: usize = x.try_into().ok()?;
        let y: usize = y.try_into().ok()?;

        if y < self.height && x < self.width {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    fn cell(&self, p: Point) -> Option<Cell> {
        let i = self.index_of(p)?;
        Some(self.cells[i])
    }

    fn cell_mut(&mut self, p: Point) -> Option<&mut Cell> {
        let i = self.index_of(p)?;
        Some(&mut self.cells[i])
    }

    fn simulation(&self) -> Simulation {
        Simulation {
            filled: false,
            grains: vec![SPAWN_POINT],
            grid: self.to_owned(),
            settled: 0,
        }
    }
}

struct Simulation {
    filled: bool,
    grains: Vec<Point>,
    grid: Grid,
    settled: usize,
}

impl Simulation {
    fn step(&mut self) -> usize {
        let mut grains = std::mem::take(&mut self.grains);

        let _ = grains
            .drain_filter(|grain| {
                if self.filled {
                    return true;
                }

                let down = *grain + Point { x: 0, y: 1 };
                let down_left = *grain + Point { x: -1, y: 1 };
                let down_right = *grain + Point { x: 1, y: 1 };
                let options = [down, down_left, down_right];

                if let Some(p) = options
                    .into_iter()
                    .find(|p| matches!(self.grid.cell(*p), Some(Cell::Air)))
                {
                    *grain = p;
                    // Keep
                    return false;
                };

                if options.into_iter().any(|p| self.grid.cell(p).is_none()) {
                    // Remove
                    return true;
                }

                if self.grid.cell(*grain) == Some(Cell::Sand) {
                    self.filled = true;
                    return false;
                }

                self.settled += 1;
                *self.grid.cell_mut(*grain).unwrap() = Cell::Sand;
                // Remove
                true
            })
            .count();

        self.grains = grains;
        if !self.filled {
            self.grains.push(SPAWN_POINT);
        }

        self.settled
    }
}

struct Task {
    polylines: Polylines,
}

impl Task {
    fn parse(input: &str) -> Result<Self> {
        let polylines = input.parse::<Polylines>()?;
        Ok(Self { polylines })
    }

    fn sand_at_rest(&self) -> usize {
        let grid = self.polylines.to_grid().unwrap();
        self.count_sand(&grid, 100)
    }

    fn sand_with_floor(&self) -> usize {
        let (xmin, xmax, _, ymax) = self.polylines.dimensions();

        // Include the floor as a very long polyline
        let floor = Polyline {
            points: vec![
                Point {
                    x: xmin - 10_000,
                    y: ymax + 2,
                },
                Point {
                    x: xmax + 10_000,
                    y: ymax + 2,
                },
            ],
        };

        let grid = self.polylines.with(floor).to_grid().unwrap();
        self.count_sand(&grid, 10_000)
    }

    fn count_sand(&self, grid: &Grid, steps: usize) -> usize {
        let mut s = grid.simulation();
        let mut curr = usize::MAX;

        // TODO: Figure out a more reliable approach to determining when to exit this loop
        while curr != s.settled {
            curr = s.settled;
            for _ in 0..steps {
                s.step();
            }
        }

        s.settled
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = Task::parse(&input)?;
    println!("settled sand: {}", task.sand_at_rest());
    println!("settled sand with floor: {}", task.sand_with_floor());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_from_str() {
        let p = Point::from_str("5,5").unwrap();
        assert_eq!(p, Point { x: 5, y: 5 });
    }

    #[test]
    fn polyline_from_str() {
        let Polyline { points } = Polyline::from_str("498,4 -> 498,6 -> 496,6").unwrap();
        assert_eq!(
            points,
            vec![
                Point { x: 498, y: 4 },
                Point { x: 498, y: 6 },
                Point { x: 496, y: 6 }
            ]
        );
    }

    #[test]
    fn path_points() {
        let p = Polyline::from_str("498,4 -> 498,6 -> 496,6").unwrap();
        assert_eq!(
            p.path_points().collect_vec(),
            vec![
                Point { x: 498, y: 4 },
                Point { x: 498, y: 5 },
                Point { x: 498, y: 6 },
                Point { x: 497, y: 6 },
                Point { x: 496, y: 6 }
            ]
        );
    }

    #[test]
    fn task1() {
        let input = include_str!("../data/example.txt");
        let task = Task::parse(input).unwrap();
        assert_eq!(task.sand_at_rest(), 24);
    }

    #[test]
    fn task1_with_input() {
        let input = include_str!("../data/input.txt");
        let task = Task::parse(input).unwrap();
        assert_eq!(task.sand_at_rest(), 979);
    }

    #[test]
    fn task2() {
        let input = include_str!("../data/example.txt");
        let task = Task::parse(input).unwrap();
        assert_eq!(task.sand_with_floor(), 93);
    }
}
