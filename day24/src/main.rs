// https://adventofcode.com/2022/day/24
//
// Solutions from https://www.reddit.com/r/adventofcode/comments/zu28ij/2022_day_24_solutions/
//  - https://www.reddit.com/r/adventofcode/comments/zu28ij/comment/j32ncvf/ (210ms)
//
use bitflags::bitflags;
use color_eyre::Result;
use std::{
    collections::VecDeque,
    fmt::{Debug, Write},
    io::{self, Read},
};

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Cell: u8 {
        const NORTH = 1 << 1;
        const EAST  = 1 << 2;
        const SOUTH = 1 << 3;
        const WEST  = 1 << 4;
        const ELF   = 1 << 5;
        const WIND  = Self::NORTH.bits() | Self::EAST.bits() | Self::SOUTH.bits() |
            Self::WEST.bits();
    }
}

impl Cell {
    fn winds(&self) -> Vec<Self> {
        let mut dirs = vec![];

        if self.contains(Self::NORTH) {
            dirs.push(Self::NORTH);
        }

        if self.contains(Self::EAST) {
            dirs.push(Self::EAST);
        }

        if self.contains(Self::SOUTH) {
            dirs.push(Self::SOUTH);
        }

        if self.contains(Self::WEST) {
            dirs.push(Self::WEST);
        }

        dirs
    }
}

#[derive(Clone, Copy, Debug)]
struct Pos(isize, isize);

impl Pos {
    const START: Self = Self(0, 0);

    fn adjacent(&self) -> impl Iterator<Item = Self> + '_ {
        [(-1, 0), (0, 1), (1, 0), (0, -1)]
            .iter()
            .map(|(di, dj)| Self(self.0 + di, self.1 + dj))
    }
}

#[derive(Default, Clone)]
struct State {
    up: VecDeque<u128>,
    down: VecDeque<u128>,
    left: Vec<u128>,
    right: Vec<u128>,
    width: isize,
    height: isize,
    queue: VecDeque<Pos>,
    minutes_passed: usize,
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('#')?;
        for j in 0..self.width {
            if j == 0 {
                if self.queue.is_empty() {
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
                let cell = self.value_at(i, j);
                let winds = cell.winds();

                if !winds.is_empty() {
                    if winds.len() == 1 {
                        let c = match winds[0] {
                            Cell::NORTH => '^',
                            Cell::EAST => '>',
                            Cell::SOUTH => 'v',
                            Cell::WEST => '<',
                            _ => panic!("unknown wind: {:?}", winds),
                        };
                        f.write_char(c)?;
                    } else {
                        let n = winds.len() as u8;
                        f.write_char(n as char)?;
                    }
                } else if cell == Cell::ELF {
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
    fn reached_exit(&self) -> bool {
        self.queue
            .iter()
            .any(|&Pos(i, j)| i == self.height - 1 && j == self.width - 1)
    }

    fn tick(&mut self) {
        self.up.rotate_left(1);
        self.down.rotate_right(1);
        self.minutes_passed += 1;
    }

    fn position_open(&self, _pos: &Pos) -> bool {
        false
    }

    fn value_at(&self, _i: isize, _j: isize) -> Cell {
        Cell::empty()
    }

    fn step(&mut self) {
        if self.queue.is_empty() && self.position_open(&Pos::START) {
            self.queue.push_back(Pos::START);
        }

        let mut n = self.queue.len();

        while let Some(pos) = self.queue.pop_front() {
            for adj in pos.adjacent() {
                if self.position_open(&adj) {
                    self.queue.push_back(adj);
                }
            }

            n -= 1;
            if n <= 0 {
                break;
            }
        }

        self.tick();
    }
}

struct Task {
    initial_state: State,
}

impl Task {
    fn min_minutes(&self) -> usize {
        let mut state = self.initial_state.clone();

        while !state.reached_exit() {
            state.step();
        }

        state.minutes_passed
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

    println!("fewest minutes: {}", task.min_minutes());

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
        assert_eq!(normalize(expected), normalize(&actual));
    }

    #[test]
    fn parsing() {
        let input = include_str!("../data/example.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.initial_state.height, 5);
        assert_eq!(task.initial_state.width, 5);
    }

    #[test]
    fn part1() {
        let input = include_str!("../data/example.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.min_minutes(), 18);
    }

    #[test]
    fn state() {
        let input = include_str!("../data/example.txt");
        let Task {
            initial_state: state,
        } = parse(input).unwrap();

        assert_eq!(state.minutes_passed, 0);

        assert_same(
            &state,
            "#E######
             #>>.<^<#
             #.<..<<#
             #>v.><>#
             #<^v^^>#
             ######.#",
        )
    }
}
