use color_eyre::{self, Report, Result};
use std::{
    fmt::{Debug, Write},
    str::FromStr,
};

type Int = i64;
type Point = (Int, Int);

const CHAMBER_WIDTH: usize = 7;

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

pub struct Row([Cell; CHAMBER_WIDTH]);

impl Row {
    const EMPTY: [Cell; CHAMBER_WIDTH] = [Cell::Empty; CHAMBER_WIDTH];

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
    pub max_i_by_col: [Int; CHAMBER_WIDTH],
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

impl Default for Chamber {
    fn default() -> Self {
        Self {
            rows: Vec::with_capacity(4096),
            max_i_by_col: [-1; CHAMBER_WIDTH],
            max_i: -1,
        }
    }
}

impl Chamber {
    fn is_available(&self, p: Point) -> bool {
        let (i, j) = p;

        if j < 0 {
            return false;
        }
        let j = j as usize;

        if j >= CHAMBER_WIDTH {
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct StateKey {
    relative_heights: [Int; CHAMBER_WIDTH],
    rock_index: usize,
    jet: usize,
}

#[derive(Debug)]
struct StateValue {
    max_i: Int,
    total_rocks: Int,
}

#[derive(Debug, Default)]
struct State {
    rock: Option<Rock>,
    curr_rock: usize,
    jet: usize,
    num_rocks: usize,
    chamber: Chamber,
    skipped_cycles: Int,
    cycle_found: bool,
    jets: Vec<Direction>,
    height_gain_in_cycle: Int,
    cycle_states: rustc_hash::FxHashMap<StateKey, StateValue>,
}

impl State {
    const NUM_SHAPES: usize = 5;

    const SHAPES: [Shape; Self::NUM_SHAPES] = [
        Shape::Horizontal,
        Shape::Plus,
        Shape::ReverseL,
        Shape::Vertical,
        Shape::Square,
    ];

    fn new(num_rocks: usize, jets: Vec<Direction>) -> Self {
        Self {
            num_rocks,
            jets,
            ..Default::default()
        }
    }

    fn height(&self) -> Int {
        self.chamber.max_i + (self.skipped_cycles * self.height_gain_in_cycle) + 1
    }

    fn next(mut self) -> Self {
        let num_jets = self.jets.len();
        let rock_index = self.curr_rock % Self::NUM_SHAPES;

        let mut rock = Rock {
            shape: Self::SHAPES[rock_index],
            bottom_left: (self.chamber.max_i + 4, 2),
        };
        self.rock = Some(rock.clone());

        while rock.step(&self.chamber, self.jets[self.jet % num_jets]) {
            self.jet = (self.jet + 1) % num_jets;
        }

        self.jet = (self.jet + 1) % num_jets;
        self.curr_rock += 1;
        self.chamber.insert(rock);

        if !self.cycle_found {
            self.check_cycle(rock_index);
        }

        self
    }

    fn done(&self) -> bool {
        self.curr_rock >= self.num_rocks
    }

    fn check_cycle(&mut self, rock_index: usize) {
        let mut relative_heights = self.chamber.max_i_by_col;
        let lowest = relative_heights.iter().copied().min().unwrap();

        for h in &mut relative_heights {
            *h -= lowest;
        }

        let state_key = StateKey {
            relative_heights,
            rock_index,
            jet: self.jet,
        };

        if let Some(state_value) = self.cycle_states.get(&state_key) {
            self.height_gain_in_cycle = self.chamber.max_i - state_value.max_i;
            let rocks_in_cycle = self.curr_rock as Int - state_value.total_rocks;
            self.skipped_cycles = (self.num_rocks - self.curr_rock) as Int / rocks_in_cycle;
            self.curr_rock += (self.skipped_cycles * rocks_in_cycle) as usize;
            self.cycle_found = true;
        } else {
            self.cycle_states.insert(
                state_key,
                StateValue {
                    max_i: self.chamber.max_i,
                    total_rocks: self.curr_rock as Int,
                },
            );
        }
    }
}

pub struct Task {
    jets: Vec<Direction>,
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

        Ok(Self { jets: gusts })
    }
}

impl Task {
    pub fn height_of_tower(&self, num_rocks: usize) -> Int {
        self.state_at(num_rocks).height()
    }

    fn state_at(&self, num_rocks: usize) -> State {
        let mut state = State::new(num_rocks, self.jets.clone());

        while !state.done() {
            state = state.next();
        }

        state
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
        assert_eq!(&task.jets[0..5], &vec![R, R, R, L, L]);

        let n = task.jets.len();
        assert_eq!(&task.jets[n - 5..n], &vec![R, L, L, R, R]);
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
        let state = task.state_at(25);

        assert_eq!(state.jet, 145);
        assert_eq!(state.height(), 39);
        assert_eq!(state.rock.unwrap().shape, Shape::Square);

        let state = task.state_at(26);
        assert_eq!(state.jet, 149);
        assert_eq!(state.height(), 40);
        assert_eq!(state.rock.unwrap().shape, Shape::Horizontal);

        let state = task.state_at(27);
        assert_eq!(state.jet, 165);
        assert_eq!(state.height(), 40);
        assert_eq!(state.rock.unwrap().shape, Shape::Plus);
    }

    #[test]
    fn part2() {
        let task = EXAMPLE.parse::<Task>().unwrap();
        assert_eq!(task.height_of_tower(1_000_000_000_000), 1_514_285_714_288);
    }

    #[test]
    fn part2_with_input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.height_of_tower(1_000_000_000_000), 1_547_953_216_393);
    }
}
