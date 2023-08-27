#![feature(portable_simd)]
// Other solutions from
// https://www.reddit.com/r/adventofcode/comments/zt6xz5/2022_day_23_solutions/
//
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j2wamb3/ (4.5ms)
//    Revisit.  SIMD, bit arithmetic, MapWindowsIterator, cartesian_product,
//    u8x32, slice.rotate_left
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1ehk9y/ (76ms)
//    Revisit. Static DIRECTIONS array and iterating over t..t + 4 to get rotation,
//    type Pos = (Number, Number), concise solution using a HashSet, set.reserve(n),
//    kdam progress bar
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1f9cz2/ (35ms)
//    Bit arithmetic, grid of Row([u128; MAX_WORDS]), word.count_ones(), value.trailing_zeros().
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1dq8oj/ (135ms)
//    set.insert(value) #=> bool, concise solution similar to /comments/zt6xz5/comment/j1ehk9y/.
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1d1mod/ (160ms)
//    FxHashMap
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1cfrpn/ (355ms)
//    HashMap, map.drain(), VecDeque for directions
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1carq9/ (400ms)
//    HashSet, set.reserve(n), map of pos -> vec![proposals]
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1d2wi5/ (1000ms)
//    Hand-rolled counter, HashMap of proposals, cycle detection (?)
//
// To review:
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1co30d/ (1300ms)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1chjsl/ (5000ms)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1cuopt/ (20,000ms)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1cinb1/ (25,000ms)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1ow3uv/ (?s)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1hy780/ (?s)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1eyw80/ (?s)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1exxm6/ (?s)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1dmoft/ (?s)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1dcz3p/ (?s)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1d945r/ (?s)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1cbg9k/ (?s)
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j1cqqof/ (?s)
//
// Changes:
//  - Simplify parsing code
//  - Switch to SIMD and bit arithmetic
//
use color_eyre::Result;
use itertools::{chain, Itertools};
use std::array;
use std::collections::VecDeque;
use std::ops::Range;
use std::simd::u8x32;
use std::{
    fmt::{Debug, Write},
    io::{self, Read},
    ops::Add,
};

#[derive(Debug, Clone, Copy)]
enum Direction {
    North,
    South,
    West,
    East,
}

use Direction::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Pos(usize, usize);

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Pos {
    #[allow(unused)]
    fn new(i: usize, j: usize) -> Self {
        Self(i, j)
    }

    #[allow(unused)]
    fn coords(&self) -> (usize, usize) {
        (self.0, self.1)
    }
}

const WIDTH: usize = 160;

#[derive(Clone)]
struct BitGrid([u8x32; WIDTH]);

fn shift_west(&row: &u8x32) -> u8x32 {
    (row >> u8x32::splat(1)) | (row.rotate_lanes_left::<1>() << u8x32::splat(7))
}

fn shift_east(&row: &u8x32) -> u8x32 {
    (row << u8x32::splat(1)) | (row.rotate_lanes_right::<1>() >> u8x32::splat(7))
}

fn propose(
    [nw, n, ne]: &[u8x32; 3],
    [w, cur, e]: &[u8x32; 3],
    [sw, s, se]: &[u8x32; 3],
    priority: [Direction; 4],
) -> [u8x32; 4] {
    let mut propositions = [*cur; 4];
    let mut not_chosen = nw | n | ne | w | e | sw | s | se;
    for d in priority {
        let (row, dir_available) = match d {
            North => (&mut propositions[0], !(ne | n | nw)),
            South => (&mut propositions[1], !(se | s | sw)),
            West => (&mut propositions[2], !(nw | w | sw)),
            East => (&mut propositions[3], !(ne | e | se)),
        };
        *row &= dir_available & not_chosen;
        not_chosen &= !dir_available;
    }
    propositions
}

fn collide_proposals(
    [_, south, _, _]: &[u8x32; 4],
    [_, _, west, east]: &[u8x32; 4],
    [north, _, _, _]: &[u8x32; 4],
) -> [u8x32; 4] {
    [
        north & !*south,
        south & !*north,
        shift_west(west) & !shift_east(east),
        shift_east(east) & !shift_west(west),
    ]
}

impl Debug for BitGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (rows, cols) = self.bounds();
        for row in rows {
            for col in cols.clone() {
                if self.get(row, col) {
                    print!("#");
                } else {
                    print!(".");
                }
            }
            f.write_char('\n')?;
        }
        f.write_char('\n')
    }
}

impl BitGrid {
    fn new() -> Self {
        Self([Default::default(); WIDTH])
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn bounds(&self) -> (Range<usize>, Range<usize>) {
        let (mut min_i, mut max_i) = (usize::MAX, usize::MIN);
        let (mut min_j, mut max_j) = (usize::MAX, usize::MIN);

        for Pos(i, j) in self.iter() {
            min_i = min_i.min(i);
            max_i = max_i.max(i);
            min_j = min_j.min(j);
            max_j = max_j.max(j);
        }

        (min_i..max_i + 1, min_j..max_j + 1)
    }

    fn insert(&mut self, row: usize, col: usize) {
        self.0[row][col / 8] |= 1 << (col % 8);
    }

    fn get(&self, row: usize, col: usize) -> bool {
        self.0[row][col / 8] & (1 << (col % 8)) != 0
    }

    #[allow(unused)]
    fn dimensions(&self) -> (usize, usize) {
        let (rows, cols) = self.bounds();
        (rows.len() + 1, cols.len() + 1)
    }

    fn iter(&self) -> impl Iterator<Item = Pos> + '_ {
        (0..WIDTH)
            .cartesian_product(0..256)
            .filter(|&(row, col)| self.get(row, col))
            .map(|(i, j)| Pos(i, j))
    }
}

struct MapWindows<I: Iterator, F, T, const N: usize>
where
    F: FnMut([&I::Item; N]) -> T,
{
    iter: I,
    f: F,
    buf: VecDeque<I::Item>,
}

impl<I: Iterator, F, T, const N: usize> MapWindows<I, F, T, N>
where
    F: FnMut([&I::Item; N]) -> T,
{
    fn new(mut iter: I, f: F) -> Self {
        let buf = iter.by_ref().take(N - 1).collect();
        Self { iter, f, buf }
    }
}

impl<I: Iterator, F, T, const N: usize> Iterator for MapWindows<I, F, T, N>
where
    F: FnMut([&I::Item; N]) -> T,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|next| {
            self.buf.push_back(next);
            let res = (self.f)(array::from_fn(|i| &self.buf[i]));
            self.buf.pop_front();
            res
        })
    }
}

trait MapWindowsIterator: Iterator {
    fn map_windows<T, F, const N: usize>(self, f: F) -> MapWindows<Self, F, T, N>
    where
        Self: Sized,
        F: FnMut([&Self::Item; N]) -> T,
    {
        MapWindows::new(self, f)
    }
}

impl<I: Iterator> MapWindowsIterator for I {}

#[derive(Clone)]
struct State {
    grid: BitGrid,
    round: usize,
    moved: bool,
    priority: [Direction; 4],
}

impl State {
    fn empty_tiles(&self) -> usize {
        let (rows, cols) = self.grid.bounds();
        rows.len() * cols.len() - self.grid.len()
    }

    fn step(self) -> Self {
        let State {
            mut grid,
            mut priority,
            round,
            ..
        } = self.clone();

        let mut moved = false;
        let zeros = [Default::default(); 2];

        chain!(&zeros, &self.grid.0, &zeros)
            .map(|row| [shift_east(row), *row, shift_west(row)])
            .map_windows(|[above, cur, below]| propose(above, cur, below, priority))
            .map_windows(|[above, cur, below]| collide_proposals(above, cur, below))
            .enumerate()
            .for_each(|(i, [from_south, from_north, from_east, from_west])| {
                let destinations = from_north | from_south | from_west | from_east;
                if destinations == u8x32::splat(0) {
                    return;
                }
                moved = true;
                grid.0[i + 1] &= !from_south;
                grid.0[i - 1] &= !from_north;
                grid.0[i] &= !shift_west(&from_west);
                grid.0[i] &= !shift_east(&from_east);
                grid.0[i] |= destinations;
            });

        priority.rotate_left(1);

        Self {
            grid,
            priority,
            round: round + 1,
            moved,
        }
    }
}

struct Task {
    grid: BitGrid,
}

impl Task {
    fn part1(&self) -> usize {
        self.advance(10).empty_tiles()
    }

    fn part2(&self) -> usize {
        self.advance(100_000).round
    }

    fn advance(&self, rounds: usize) -> State {
        let mut state = self.start();

        for _ in 0..rounds {
            state = state.step();

            if !state.moved {
                return state;
            }
        }

        state
    }

    fn start(&self) -> State {
        let grid = self.grid.clone();

        State {
            round: 0,
            grid,
            moved: true,
            priority: [North, South, West, East],
        }
    }
}

fn parse(s: &str) -> Result<Task> {
    let mut grid = BitGrid::new();
    s.lines().enumerate().for_each(|(row, line)| {
        line.chars()
            .enumerate()
            .filter(|&(_, c)| c == '#')
            .for_each(|(col, _)| grid.insert(row + 24, col + 72))
    });

    Ok(Task { grid })
}

fn main() -> Result<()> {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;
    let task = parse(&s)?;

    println!("empty tiles: {}", task.part1());
    println!("number of rounds: {}", task.part2());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> &'static str {
        include_str!("../data/example.txt")
    }

    fn step(mut state: State, steps: usize) -> State {
        for _ in 0..steps {
            state = state.step();
        }
        state
    }

    fn normalize(s: &str) -> Vec<&str> {
        s.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
    }

    fn assert_same(left: &str, right: &str) {
        assert_eq!(normalize(left), normalize(right))
    }

    #[test]
    fn parsing() {
        let task = parse(example()).unwrap();
        assert_eq!(task.grid.len(), 160);
    }

    #[test]
    fn empty_tiles() {
        let task = parse(example()).unwrap();
        let mut state = task.start();

        assert_eq!(state.empty_tiles(), 27);

        state = state.step();
        assert_eq!(state.empty_tiles(), 59);
    }

    #[test]
    fn state() {
        let input = "
        .....
        ..##.
        ..#..
        .....
        ..##.
        .....";

        let task = parse(input).unwrap();
        let mut state = task.start();

        assert_same(
            &format!("{:?}", state.grid),
            "##
             #.
             ..
             ##",
        );

        state = step(state, 1);

        assert_same(
            &format!("{:?}", state.grid),
            "##
             ..
             #.
             .#
             #.",
        );

        state = step(state, 1);

        assert_same(
            &format!("{:?}", state.grid),
            ".##.
             #...
             ...#
             ....
             .#..",
        );

        state = step(state, 1);

        assert_same(
            &format!("{:?}", state.grid),
            "..#..
             ....#
             #....
             ....#
             .....
             ..#..",
        );
    }

    #[test]
    fn part1() {
        let task = parse(example()).unwrap();
        let mut state = task.start();

        // Start
        assert_same(
            &format!("{:?}", state.grid),
            "....#..
             ..###.#
             #...#.#
             .#...##
             #.###..
             ##.#.##
             .#..#..",
        );

        state = step(state, 1);

        // Round 1
        assert_same(
            &format!("{:?}", state.grid),
            ".....#...
             ...#...#.
             .#..#.#..
             .....#..#
             ..#.#.##.
             #..#.#...
             #.#.#.##.
             .........
             ..#..#...",
        );

        state = step(state, 1);

        // Round 2
        assert_same(
            &format!("{:?}", state.grid),
            "......#....
             ...#.....#.
             ..#..#.#...
             ......#...#
             ..#..#.#...
             #...#.#.#..
             ...........
             .#.#.#.##..
             ...#..#....",
        );

        state = step(state, 1);

        // Round 3
        assert_same(
            &format!("{:?}", state.grid),
            "......#....
             ....#....#.
             .#..#...#..
             ......#...#
             ..#..#.#...
             #..#.....#.
             ......##...
             .##.#....#.
             ..#........
             ......#....",
        );

        state = step(state, 1);

        // Round 4
        assert_same(
            &format!("{:?}", state.grid),
            "......#....
             .....#....#
             .#...##....
             ..#.....#.#
             ........#..
             #...###..#.
             .#......#..
             ...##....#.
             ...#.......
             ......#....",
        );

        state = step(state, 1);

        // Round 5
        assert_same(
            &format!("{:?}", state.grid),
            "......#....
             ...........
             .#..#.....#
             ........#..
             .....##...#
             #.#.####...
             ..........#
             ...##..#...
             .#.........
             .........#.
             ...#..#....",
        );

        state = step(state, 5);

        // Round 10
        assert_same(
            &format!("{:?}", state.grid),
            "......#.....
             ..........#.
             .#.#..#.....
             .....#......
             ..#.....#..#
             #......##...
             ....##......
             .#........#.
             ...#.#..#...
             ............
             ...#..#..#..",
        );

        assert_eq!(state.round, 10);

        let (height, width) = state.grid.dimensions();
        assert_eq!(12, width);
        assert_eq!(11, height);
        assert_eq!(state.empty_tiles(), 110);

        let state = step(state, 9);
        assert_eq!(state.round, 19);

        // Round 20
        assert_same(
            &format!("{:?}", state.grid),
            ".......#......
             ....#......#..
             ..#.....#.....
             ......#.......
             ...#....#.#..#
             #.............
             ....#.....#...
             ..#.....#.....
             ....#.#....#..
             .........#....
             ....#......#..
             .......#......",
        );
    }

    #[test]
    fn input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();
        let part1 = task.part1();

        assert!(part1 < 18778);
        assert!(part1 < 4372);
        assert_eq!(part1, 4288);

        let part2 = task.part2();
        assert!(part2 > 939);
        assert_eq!(part2, 940);
    }
}
