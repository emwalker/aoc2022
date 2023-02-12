use color_eyre::{self, eyre::eyre, Result};
use itertools::Itertools;
use std::fmt::Debug;
use std::i32::{MAX, MIN};
use std::io::{self, Read};

mod parser;
use parser::{Cave, Wall};

impl Wall {
    fn contains(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }

        // (x1 - x0) * (y1 - y)  == (x1 - x) * (y1 - y0)
        (self.p1.x - self.p0.x) * (self.p1.y - y) == (self.p1.x - x) * (self.p1.y - self.p0.y)
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        self.xrange.contains(&x) && self.yrange.contains(&y)
    }
}

#[derive(Debug)]
struct State {
    cave: Cave,
    height: usize,
    width: usize,
    xmin: i32,
    ymin: i32,
}

impl State {
    fn new(cave: Cave) -> Result<Self> {
        let (mut xmin, mut xmax) = (MAX, MIN);
        let (ymin, mut ymax) = (0, MIN);

        for Wall { xrange, yrange, .. } in cave.iter() {
            xmin = xmin.min(*xrange.start());
            xmax = xmax.max(*xrange.end());
            ymax = ymax.max(*yrange.end());
        }

        xmin = xmin.min(500);
        xmax = xmax.max(500);

        if xmin == MAX || ymin == MAX || xmax == MIN || ymax == MIN {
            return Err(eyre!("failed to determine area of scan"));
        }

        let height = ymax - ymin + 1;
        let width = xmax - xmin + 1;

        if height < 1 || width < 1 {
            return Err(eyre!("bad height or width: ({height}, {width})"));
        }

        Ok(Self {
            cave,
            height: height as usize,
            width: width as usize,
            xmin,
            ymin,
        })
    }

    fn iter(&self) -> SnapshotIter {
        SnapshotIter {
            falling_sand: false,
            prev: None,
            prev_sand_at_rest: None,
            state: self,
        }
    }

    fn wall_at(&self, i: usize, j: usize) -> bool {
        let (x, y) = self.coords(i, j);
        self.cave.iter().any(|w| w.contains(x, y))
    }

    fn coords(&self, i: usize, j: usize) -> (i32, i32) {
        (j as i32 + self.xmin, i as i32 + self.ymin)
    }

    fn source_at(&self, i: usize, j: usize) -> bool {
        let (x, y) = self.coords(i, j);
        x == 500 && y == 0
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Type {
    Air,
    FallingSand,
    Rock,
    SandAtRest,
    Source,
}

impl Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Air => ".",
            Self::FallingSand => "o",
            Self::Rock => "#",
            Self::SandAtRest => "^",
            Self::Source => "+",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Eq, PartialEq)]
struct Grid(Vec<Vec<Type>>);

impl Grid {
    fn get_mut(&mut self, i: usize, j: usize) -> Option<&mut Type> {
        if let Some(row) = self.0.get_mut(i) {
            if let Some(cell) = row.get_mut(j) {
                return Some(cell);
            }
        }
        None
    }

    fn iter(&self) -> impl Iterator<Item = &Vec<Type>> + '_ {
        self.0.iter()
    }

    fn at_cell(&self, i: usize, j: usize) -> Option<Square> {
        self.0.get(i).and_then(|row| {
            Some(Square {
                i,
                j,
                square_type: *row.get(j)?,
            })
        })
    }

    fn above(&self, s: &Square) -> Option<Square> {
        self.at_cell(s.i.checked_sub(1)?, s.j)
    }

    fn above_right(&self, s: &Square) -> Option<Square> {
        self.at_cell(s.i.checked_sub(1)?, s.j + 1)
    }

    fn right(&self, s: &Square) -> Option<Square> {
        self.at_cell(s.i, s.j + 1)
    }

    fn below(&self, s: &Square) -> Option<Square> {
        self.at_cell(s.i + 1, s.j)
    }

    fn above_left(&self, s: &Square) -> Option<Square> {
        self.at_cell(s.i.checked_sub(1)?, s.j.checked_sub(1)?)
    }

    fn left(&self, s: &Square) -> Option<Square> {
        self.at_cell(s.i, s.j.checked_sub(1)?)
    }
}

#[derive(Clone)]
struct Square {
    i: usize,
    j: usize,
    square_type: Type,
}

impl Square {
    fn of(&self, square_type: Type) -> Self {
        Square {
            i: self.i,
            j: self.j,
            square_type,
        }
    }

    fn is_falling_sand(&self) -> bool {
        self.square_type == Type::FallingSand
    }

    fn changes(&self, grid: &Grid, falling_sand: bool) -> Vec<Self> {
        self.solid_below(grid)
            .or_else(|| self.bottom_of_grid(grid))
            .or_else(|| self.falling_sand(grid))
            .or_else(|| self.down_left(grid))
            .or_else(|| self.down_right(grid))
            .or_else(|| self.emit_sand(grid, falling_sand))
            .unwrap_or_default()
    }

    fn falling_sand(&self, grid: &Grid) -> Option<Vec<Self>> {
        let above = grid.above(self)?;
        match (above.square_type, self.square_type) {
            (Type::Air, Type::FallingSand) => Some(vec![self.of(Type::Air)]),
            (Type::FallingSand, Type::Air) => {
                Some(vec![above.of(Type::Air), self.of(Type::FallingSand)])
            }
            (Type::Source, Type::FallingSand) => Some(vec![self.of(Type::Air)]),
            _ => None,
        }
    }

    fn solid_below(&self, grid: &Grid) -> Option<Vec<Self>> {
        let below = grid.below(self)?;
        match (self.square_type, below.square_type) {
            (Type::FallingSand, Type::Rock) => Some(vec![self.of(Type::SandAtRest)]),
            (Type::FallingSand, Type::SandAtRest) => Some(vec![self.of(Type::SandAtRest)]),
            _ => None,
        }
    }

    fn down_left(&self, grid: &Grid) -> Option<Vec<Self>> {
        self.down_diag(grid.above_right(self)?, grid.right(self)?)
    }

    fn down_right(&self, grid: &Grid) -> Option<Vec<Self>> {
        self.down_diag(grid.above_left(self)?, grid.left(self)?)
    }

    fn down_diag(&self, above_diag: Square, adjacent: Square) -> Option<Vec<Self>> {
        match (
            above_diag.square_type,
            adjacent.square_type,
            self.square_type,
        ) {
            (Type::FallingSand, Type::Rock, Type::Air) => {
                Some(vec![above_diag.of(Type::Air), self.of(Type::FallingSand)])
            }

            (Type::FallingSand, Type::SandAtRest, Type::Air) => {
                Some(vec![above_diag.of(Type::Air), self.of(Type::FallingSand)])
            }

            _ => None,
        }
    }

    fn bottom_of_grid(&self, grid: &Grid) -> Option<Vec<Self>> {
        let below = grid.below(self);
        match (self.square_type, below) {
            (Type::FallingSand, None) => Some(vec![self.of(Type::Air)]),
            _ => None,
        }
    }

    fn emit_sand(&self, grid: &Grid, falling_sand: bool) -> Option<Vec<Self>> {
        if falling_sand {
            return None;
        }

        let above = grid.above(self)?;
        match (above.square_type, self.square_type) {
            (Type::Source, Type::Air) => Some(vec![self.of(Type::FallingSand)]),
            _ => None,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
struct Snapshot {
    grid: Grid,
}

impl Snapshot {
    fn squares(&self) -> impl Iterator<Item = Square> + '_ {
        self.grid.iter().enumerate().flat_map(move |(i, row)| {
            row.iter()
                .enumerate()
                .map(move |(j, &square_type)| Square { i, j, square_type })
        })
    }

    fn update(&mut self, changes: Vec<Square>) {
        for s in changes.into_iter() {
            if let Some(square_type) = self.grid.get_mut(s.i, s.j) {
                *square_type = s.square_type;
            }
        }
    }

    fn sand_at_rest(&self) -> usize {
        self.squares()
            .filter(|s| s.square_type == Type::SandAtRest)
            .count()
    }
}

struct SnapshotIter<'s> {
    falling_sand: bool,
    prev_sand_at_rest: Option<usize>,
    prev: Option<Snapshot>,
    state: &'s State,
}

impl<'s> Iterator for SnapshotIter<'s> {
    type Item = Snapshot;

    fn next(&mut self) -> Option<Self::Item> {
        let State { height, width, .. } = self.state;
        let mut curr;

        if let Some(prev) = &self.prev {
            curr = prev.clone();
            let mut falling_sand = false;
            let squares = curr.squares().collect_vec();

            for square in squares {
                let changes = square.changes(&curr.grid, self.falling_sand);
                if changes.iter().any(Square::is_falling_sand) {
                    falling_sand = true;
                }
                curr.update(changes);
            }

            self.falling_sand = falling_sand;
        } else {
            let mut grid = vec![vec![Type::Air; *width]; *height];
            for (i, row) in grid.iter_mut().enumerate() {
                for (j, square_type) in row.iter_mut().enumerate() {
                    if self.state.wall_at(i, j) {
                        *square_type = Type::Rock;
                    } else if self.state.source_at(i, j) {
                        *square_type = Type::Source;
                    }
                }
            }
            curr = Snapshot { grid: Grid(grid) };
        }

        if let Some(count) = self.prev_sand_at_rest {
            let curr_count = curr.sand_at_rest();
            if curr_count == count {
                return None;
            }
        }

        if let Some(prev) = &self.prev {
            self.prev_sand_at_rest = Some(prev.sand_at_rest());
        }

        self.prev = Some(curr);
        self.prev.clone()
    }
}

impl Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.grid.iter() {
            for s in row.iter() {
                f.write_fmt(format_args!("{s:?}"))?;
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Task(State);

impl Task {
    fn parse(input: &str) -> Result<Self> {
        let cave = parser::parse(input)?;
        let state = State::new(cave)?;
        Ok(Self(state))
    }

    fn sand_at_rest(&self) -> usize {
        let snapshot = self.0.iter().take(50000).last().unwrap();
        snapshot.sand_at_rest()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = Task::parse(&input)?;
    println!("sand at rest: {}", task.sand_at_rest());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use parser::Point;

    fn task(input: &str) -> Task {
        Task::parse(input).unwrap()
    }

    #[test]
    fn scan() {
        let task = task(include_str!("../data/example.txt"));
        assert_eq!(task.0.height, 10);
        assert_eq!(task.0.width, 10);
    }

    #[test]
    fn wall() {
        let wall = Wall::new(Point { x: 0, y: 0 }, Point { x: 3, y: 3 }).unwrap();
        assert!(wall.contains(1, 1));
        assert!(!wall.contains(1, 2));
    }

    fn same(s1: &str, s2: &str) -> bool {
        s1.lines().map(|l| l.trim()).collect_vec() == s2.lines().map(|l| l.trim()).collect_vec()
    }

    fn assert_snapshot(snapshot: Snapshot, expected: &str) {
        let out = format!("{snapshot:?}");
        assert!(same(&out, expected), "\nexpected\n{expected},\ngot\n{out}");
    }

    #[test]
    fn snapshot() {
        let task = task(include_str!("../data/example.txt"));
        let mut it = task.0.iter();

        assert_snapshot(
            it.next().unwrap(),
            "......+...
             ..........
             ..........
             ..........
             ....#...##
             ....#...#.
             ..###...#.
             ........#.
             ........#.
             #########.",
        );

        assert_snapshot(
            it.next().unwrap(),
            "......+...
             ..........
             ..........
             ..........
             ....#...##
             ....#...#.
             ..###...#.
             ........#.
             ......o.#.
             #########.",
        );

        assert_snapshot(
            it.next().unwrap(),
            "......+...
             ..........
             ..........
             ..........
             ....#...##
             ....#...#.
             ..###...#.
             ........#.
             ......^.#.
             #########.",
        );

        assert_snapshot(
            it.next().unwrap(),
            "......+...
             ..........
             ..........
             ..........
             ....#...##
             ....#...#.
             ..###...#.
             ........#.
             .....o^.#.
             #########.",
        );

        assert_snapshot(
            it.next().unwrap(),
            "......+...
             ..........
             ..........
             ..........
             ....#...##
             ....#...#.
             ..###...#.
             ........#.
             .....^^.#.
             #########.",
        );

        assert_snapshot(
            it.next().unwrap(),
            "......+...
             ..........
             ..........
             ..........
             ....#...##
             ....#...#.
             ..###...#.
             ........#.
             .....^^o#.
             #########.",
        );

        assert_snapshot(
            it.next().unwrap(),
            "......+...
             ..........
             ..........
             ..........
             ....#...##
             ....#...#.
             ..###...#.
             ........#.
             .....^^^#.
             #########.",
        );

        assert_snapshot(
            it.next().unwrap(),
            "......+...
             ..........
             ..........
             ..........
             ....#...##
             ....#...#.
             ..###...#.
             ......o.#.
             .....^^^#.
             #########.",
        );

        assert_snapshot(
            it.next().unwrap(),
            "......+...
             ..........
             ..........
             ..........
             ....#...##
             ....#...#.
             ..###...#.
             ......^.#.
             .....^^^#.
             #########.",
        );

        assert_snapshot(
            it.next().unwrap(),
            "......+...
             ..........
             ..........
             ..........
             ....#...##
             ....#...#.
             ..###...#.
             ......^.#.
             ....o^^^#.
             #########.",
        );
    }

    #[test]
    fn sand_falls_through() {
        let input = "501,1 -> 501,2";

        let task = task(input);
        let mut it = task.0.iter();

        assert_snapshot(
            it.next().unwrap(),
            "+.
             .#
             .#",
        );

        assert_snapshot(
            it.next().unwrap(),
            "+.
             .#
             o#",
        );
    }
}
