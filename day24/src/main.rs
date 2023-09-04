// https://adventofcode.com/2022/day/24
//
// Solutions from https://www.reddit.com/r/adventofcode/comments/zu28ij/2022_day_24_solutions/
//  - https://www.reddit.com/r/adventofcode/comments/zu28ij/comment/j32ncvf/ (210ms)
//
use color_eyre::Result;
use itertools::izip;
use std::{
    collections::VecDeque,
    fmt::Debug,
    io::{self, Read},
};

#[derive(Clone)]
struct State {
    down: VecDeque<u128>,
    height: usize,
    left: Vec<u128>,
    right: Vec<u128>,
    up: VecDeque<u128>,
    width: usize,
}

#[derive(Debug, Clone, Copy)]
enum Destination {
    Exit,
    Entrance,
}

use Destination::*;

fn mask(width: usize) -> u128 {
    (1 << width) - 1
}

impl State {
    fn tick(&mut self) {
        self.up.rotate_left(1);
        self.down.rotate_right(1);

        self.left.iter_mut().for_each(|row| {
            *row = (*row >> 1) | ((*row & 1) << (self.width - 1));
        });
        self.right.iter_mut().for_each(|row| {
            // Q: Why are we shifting left to represent winds moving to the right?
            // A: Because we're treating (0, 0) as 1 (no left shifting) at row 0.
            *row = (*row << 1) | (*row >> (self.width - 1));
            // Q: Why do we need the mask here, but not in the previous case?
            // A: Perhaps because the map is left-justified.
            *row &= mask(self.width);
        });
    }

    fn travel_to(&mut self, dest: Destination) -> usize {
        assert_eq!(self.height, self.right.len());
        let mut positions = vec![0; self.height];

        for minute in 1.. {
            self.tick();
            positions = self.possible_moves(&positions);

            // Add an elf at the beginning or end of the maze to start the walk again in case other
            // possible moves are eliminated during the round.  This move will also be checked
            // against the wind patterns and may itself be eliminated during the round.
            match dest {
                Exit => positions[0] |= 1,
                Entrance => positions[self.height - 1] |= 1 << (self.width - 1),
            }

            let it = izip!(
                &mut positions,
                &self.up,
                &self.down,
                &self.left,
                &self.right
            );

            // Eliminate possible moves that are incomptabile with the wind patterns.
            for (row, up, down, left, right) in it {
                *row &= !(up | down | left | right);
            }

            if self.reached_destination(&positions, dest) {
                self.tick();
                return minute + 1;
            }
        }

        unreachable!()
    }

    fn reached_destination(&self, positions: &[u128], dest: Destination) -> bool {
        matches!(dest, Exit) && positions[self.height - 1] >> (self.width - 1) == 1
            || matches!(dest, Entrance) && positions[0] & 1 == 1
    }

    fn possible_moves(&self, positions: &[u128]) -> Vec<u128> {
        let mut new_positions = vec![0; self.height];

        let it = izip!(
            &mut new_positions,
            [0].iter().chain(positions),
            positions,
            positions.iter().skip(1).chain([0].iter())
        );

        for (row, above, cur, bellow) in it {
            *row = (cur | cur << 1 | cur >> 1 | above | bellow) & mask(self.width);
        }

        new_positions
    }
}

struct Task {
    initial_state: State,
}

impl Task {
    fn part1(&self) -> usize {
        let mut state = self.initial_state.clone();
        state.travel_to(Exit)
    }

    fn part2(&self) -> usize {
        let mut state = self.initial_state.clone();
        let mut minutes = state.travel_to(Exit);
        minutes += state.travel_to(Entrance);
        minutes += state.travel_to(Exit);
        minutes
    }
}

fn parse(s: &str) -> Result<Task> {
    let width = s.find('\n').expect("a newline") - 2;

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
        height: up.len(),
        up,
        down,
        left,
        right,
        width,
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
        assert_eq!(part2, 853);
    }
}
