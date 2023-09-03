// https://adventofcode.com/2022/day/24
//
// Solutions from https://www.reddit.com/r/adventofcode/comments/zu28ij/2022_day_24_solutions/
//  - https://www.reddit.com/r/adventofcode/comments/zu28ij/comment/j32ncvf/ (210ms)
//
use bitflags::bitflags;
use color_eyre::Result;
use std::{
    collections::{HashSet, VecDeque},
    fmt::{Debug, Write},
    io::{self, Read},
};

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Cell: u8 {
        const UP    = 1 << 1;
        const RIGHT = 1 << 2;
        const DOWN  = 1 << 3;
        const LEFT  = 1 << 4;
        const ELF   = 1 << 5;
        const WIND  = Self::UP.bits() | Self::RIGHT.bits() | Self::DOWN.bits() |
            Self::LEFT.bits();
    }
}

impl Cell {
    fn winds(&self) -> Vec<Self> {
        let mut dirs = vec![];

        [Self::UP, Self::RIGHT, Self::DOWN, Self::LEFT]
            .iter()
            .for_each(|&dir| {
                if self.contains(dir) {
                    dirs.push(dir)
                }
            });

        dirs
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
struct Pos(isize, isize);

impl Pos {
    const ORIGIN: Self = Self(0, 0);

    fn adjacent(&self) -> impl Iterator<Item = Self> + '_ {
        [(-1, 0), (0, 1), (1, 0), (0, -1), (0, 0)]
            .iter()
            .map(|(di, dj)| Self(self.0 + di, self.1 + dj))
    }
}

#[derive(Default, Clone)]
struct State {
    destination: Pos,
    down: VecDeque<u128>,
    elves: HashSet<Pos>,
    height: isize,
    left: Vec<u128>,
    minutes_passed: usize,
    right: Vec<u128>,
    start: Option<Pos>,
    up: VecDeque<u128>,
    width: isize,
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('#')?;
        for j in 0..self.width {
            if j == 0 {
                if self.elves.is_empty() {
                    f.write_char('E')?;
                } else {
                    f.write_char('.')?;
                }
            } else {
                f.write_char('#')?;
            }
        }
        f.write_char('#')?;
        f.write_char('\n')?;

        for i in 0..self.height {
            f.write_char('#')?;

            for j in 0..self.width {
                let pos = Pos(i, j);
                let cell = self.contents(&pos);
                let winds = cell.winds();

                if !winds.is_empty() {
                    if winds.len() == 1 {
                        let c = match winds[0] {
                            Cell::UP => '^',
                            Cell::RIGHT => '>',
                            Cell::DOWN => 'v',
                            Cell::LEFT => '<',
                            _ => panic!("unknown wind: {:?}", winds),
                        };
                        f.write_char(c)?;
                    } else {
                        let n = winds.len() as u8;
                        write!(f, "{}", n)?;
                    }
                } else if self.elves.contains(&pos) {
                    f.write_char('E')?;
                } else {
                    f.write_char('.')?;
                }
            }

            f.write_char('#')?;
            f.write_char('\n')?;
        }

        f.write_char('#')?;
        for j in 0..self.width {
            if j == self.width - 1 {
                f.write_char('.')?;
            } else {
                f.write_char('#')?;
            }
        }
        f.write_char('#')?;
        f.write_char('\n')
    }
}

impl State {
    fn arrived(&self) -> bool {
        let Pos(dest_i, dest_j) = self.destination;
        self.elves
            .iter()
            .any(|&Pos(i, j)| i == dest_i && j == dest_j)
    }

    fn tick(&mut self) {
        self.up.rotate_left(1);
        self.down.rotate_right(1);
        self.minutes_passed += 1;
    }

    fn position_open(&self, pos: &Pos) -> bool {
        if pos.0 < 0 || pos.0 >= self.height {
            return false;
        }

        if pos.1 < 0 || pos.1 >= self.width {
            return false;
        }

        self.contents(pos) == Cell::empty()
    }

    fn contents(&self, &Pos(i, j): &Pos) -> Cell {
        debug_assert!(i >= 0 && i < self.height);
        debug_assert!(j >= 0 && i < self.width);

        let i = i as usize;
        let mut cell = Cell::empty();

        // Left
        let j_left = 1 << (j + self.minutes_passed as isize).rem_euclid(self.width) as u128;
        let winds_left = self.left[i];
        if (winds_left & j_left) == j_left {
            cell |= Cell::LEFT;
        }

        // Right
        let j_right = 1 << (j - self.minutes_passed as isize).rem_euclid(self.width) as u128;
        let winds_right = self.right[i];
        if (winds_right & j_right) == j_right {
            cell |= Cell::RIGHT;
        }

        let j = 1 << j as u128;

        // Up
        let winds_up = *self.up.get(i).expect("upward wind");
        if (winds_up & j) == j {
            cell |= Cell::UP;
        }

        // Down
        let winds_down = *self.down.get(i).expect("downward wind");
        if (winds_down & j) == j {
            cell |= Cell::DOWN;
        }

        cell
    }

    fn step(&mut self) {
        self.tick();

        if self.elves.is_empty() {
            if let Some(start) = self.start {
                if self.position_open(&start) {
                    self.elves.insert(start);
                }
            }
        }

        let elves = self.elves.drain().collect::<Vec<_>>();

        for pos in elves {
            for adj in pos.adjacent() {
                if self.position_open(&adj) {
                    self.elves.insert(adj);
                }
            }
        }
    }

    fn maze_exit(&self) -> Pos {
        Pos(self.height - 1, self.width - 1)
    }

    fn configure(&mut self, start: Pos, destination: Pos) -> &mut Self {
        self.elves.clear();
        self.start = Some(start);
        self.destination = destination;
        self
    }

    fn travel(&mut self) -> usize {
        while !self.arrived() {
            self.step();
        }
        self.minutes_passed + 1
    }
}

struct Task {
    initial_state: State,
}

impl Task {
    fn part1(&self) -> usize {
        let mut state = self.initial_state.clone();
        let exit = state.maze_exit();
        state.configure(Pos::ORIGIN, exit).travel()
    }

    fn part2(&self) -> usize {
        let mut state = self.initial_state.clone();
        let exit = state.maze_exit();
        state.configure(Pos::ORIGIN, exit).travel();
        state.configure(exit, Pos::ORIGIN).travel();
        state.configure(Pos::ORIGIN, exit).travel()
    }
}

fn parse(s: &str) -> Result<Task> {
    let width = (s.find('\n').expect("a newline") - 2) as isize;

    let (up, (down, (left, right))): (VecDeque<_>, (VecDeque<_>, (Vec<_>, Vec<_>))) = s
        .lines()
        .filter(|line| &line[2..3] != "#")
        .map(|line| {
            let (mut up, mut down, mut left, mut right) = (0, 0, 0, 0);

            line.bytes()
                .filter(|&c| c != b'#')
                .enumerate()
                .for_each(|(j, c)| {
                    let bit = 1 << j;
                    match c {
                        b'^' => up |= bit,
                        b'>' => right |= bit,
                        b'v' => down |= bit,
                        b'<' => left |= bit,
                        _ => {}
                    };
                });

            (up, (down, (left, right)))
        })
        .unzip();

    let initial_state = State {
        height: up.len() as isize,
        up,
        down,
        left,
        right,
        width,
        ..Default::default()
    };

    Ok(Task { initial_state })
}

fn main() -> Result<()> {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;
    let task = parse(&s)?;

    println!("minutes to exit: {}", task.part1());
    println!("after getting snacks: {}", task.part2());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    fn normalize(s: &str) -> String {
        s.trim().lines().map(|l| l.trim()).join("\n")
    }

    fn assert_same(state: &State, expected: &str) {
        let actual = format!("{:?}", state);
        assert_eq!(normalize(&actual), normalize(expected));
    }

    #[test]
    fn parsing() {
        let input = include_str!("../data/example.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.initial_state.height, 4);
        assert_eq!(task.initial_state.width, 6);
    }

    #[test]
    fn part1() {
        let input = include_str!("../data/example.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.part1(), 18);
    }

    #[test]
    fn part2() {
        let input = include_str!("../data/example.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.part2(), 54);
    }

    #[test]
    fn input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();

        let part1 = task.part1();
        assert_eq!(part1, 266);

        let part2 = task.part2();
        assert!(part2 > 805);
        assert_eq!(part2, 803);
    }

    #[test]
    fn state() {
        let input = include_str!("../data/example.txt");
        let Task {
            initial_state: mut state,
        } = parse(input).unwrap();
        let exit = state.maze_exit();
        state.configure(Pos::ORIGIN, exit);

        assert_eq!(state.minutes_passed, 0);
        assert_same(
            &state,
            "#E######
             #>>.<^<#
             #.<..<<#
             #>v.><>#
             #<^v^^>#
             ######.#",
        );

        state.step();

        assert_eq!(state.minutes_passed, 1);
        assert_same(
            &state,
            "#.######
             #E>3.<.#
             #<..<<.#
             #>2.22.#
             #>v..^<#
             ######.#",
        );

        state.step();

        assert_eq!(state.minutes_passed, 2);
        assert_same(
            &state,
            "#.######
             #E2>2..#
             #E^22^<#
             #.>2.^>#
             #.>..<.#
             ######.#",
        );

        state.step();

        assert_eq!(state.minutes_passed, 3);
        assert_same(
            &state,
            "#.######
             #<^<22.#
             #E2<.2.#
             #><2>..#
             #..><..#
             ######.#",
        );

        state.step();

        assert_eq!(state.minutes_passed, 4);
        assert_same(
            &state,
            "#.######
             #E<..22#
             #<<.<..#
             #<2.>>.#
             #.^22^.#
             ######.#",
        );

        state.step();

        assert_eq!(state.minutes_passed, 5);
        assert_same(
            &state,
            "#.######
             #2Ev.<>#
             #<.<..<#
             #.^>^22#
             #.2..2.#
             ######.#",
        );
    }
}
