// https://adventofcode.com/2022/day/23
//
// Other solutions from
// https://www.reddit.com/r/adventofcode/comments/zt6xz5/2022_day_23_solutions/
//
//  - https://www.reddit.com/r/adventofcode/comments/zt6xz5/comment/j2wamb3/ (4.5ms)
//    Revisit.  SIMD, bit arithmetic, MapWindowsIterator, cartesian_product,
//    SimdVec, slice.rotate_left
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
#![feature(portable_simd)]
use auto_ops::impl_op_ex;
use color_eyre::Result;
use itertools::{chain, Itertools};
use std::array;
use std::collections::VecDeque;
use std::ops::IndexMut;
use std::simd;
use std::{
    fmt::{Debug, Write},
    io::{self, Read},
    ops::{Add, BitAnd, BitAndAssign, BitOrAssign, Index, Range},
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

// Since the elves expand out from their initial position, you need a wide enough row to accomodate
// the expansion.  In the case of the inputs provided, u8x16 is not wide enough.
type SimdVec = simd::u8x32;
const BITS_PER_ROW: usize = 8 * SimdVec::LANES; // 256
const NUM_ROWS: usize = 160;

#[derive(Clone, Copy, Default)]
struct Row(SimdVec);

impl_op_ex!(!|a: &Row| -> Row { Row(!a.0) });
impl_op_ex!(| |a: &Row, b: &Row | -> Row { Row(a.0 | b.0) });

impl BitAnd for Row {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Row {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitOrAssign for Row {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl Row {
    fn shift_west(&self) -> Self {
        Self((self.0 >> SimdVec::splat(1)) | (self.0.rotate_lanes_left::<1>() << SimdVec::splat(7)))
    }

    fn shift_east(&self) -> Self {
        Self(
            (self.0 << SimdVec::splat(1)) | (self.0.rotate_lanes_right::<1>() >> SimdVec::splat(7)),
        )
    }
}

impl Index<usize> for Row {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Row {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Row {
    fn is_empty(&self) -> bool {
        self.0 == SimdVec::splat(0)
    }
}

#[derive(Clone)]
struct BitGrid([Row; NUM_ROWS]);

struct Proposal([Row; 4]);

impl Proposal {
    fn propose(
        [nw, n, ne]: &[Row; 3],
        [w, cur, e]: &[Row; 3],
        [sw, s, se]: &[Row; 3],
        priority: [Direction; 4],
    ) -> Self {
        let mut proposals = [*cur; 4];
        let mut not_chosen = nw | n | ne | w | e | sw | s | se;
        for d in priority {
            let (row, dir_available) = match d {
                North => (&mut proposals[0], !(ne | n | nw)),
                South => (&mut proposals[1], !(se | s | sw)),
                West => (&mut proposals[2], !(nw | w | sw)),
                East => (&mut proposals[3], !(ne | e | se)),
            };
            *row &= dir_available & not_chosen;
            not_chosen &= !dir_available;
        }
        Self(proposals)
    }

    fn collide(
        Self([_, south, _, _]): &Self,
        Self([_, _, west, east]): &Self,
        Self([north, _, _, _]): &Self,
    ) -> Self {
        Self([
            *north & !*south,
            *south & !*north,
            west.shift_west() & !east.shift_east(),
            east.shift_east() & !west.shift_west(),
        ])
    }
}

impl Debug for BitGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (rows, cols) = self.bounds();
        for i in rows {
            for j in cols.clone() {
                if self.has_elf(i, j) {
                    f.write_char('#')?;
                } else {
                    f.write_char('.')?;
                }
            }
            f.write_char('\n')?;
        }
        f.write_char('\n')
    }
}

impl Default for BitGrid {
    fn default() -> Self {
        Self([Default::default(); NUM_ROWS])
    }
}

impl BitGrid {
    fn new() -> Self {
        Self::default()
    }

    fn len(&self) -> usize {
        self.0
            .iter()
            .flat_map(|x| x.0.as_array())
            .map(|x| x.count_ones() as usize)
            .sum()
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

    fn insert(&mut self, i: usize, j: usize) {
        // j / 8, to get the index of the lane for j, since there are 8 bits per lane
        self.0[i][j / 8] |= 1 << (j % 8);
    }

    fn has_elf(&self, i: usize, j: usize) -> bool {
        // j / 8, to get the index of the lane for j, since there are 8 bits per lane
        self.0[i][j / 8] & (1 << (j % 8)) != 0
    }

    #[allow(unused)]
    fn dimensions(&self) -> (usize, usize) {
        let (rows, cols) = self.bounds();
        (rows.len(), cols.len())
    }

    fn iter(&self) -> impl Iterator<Item = Pos> + '_ {
        (0..NUM_ROWS)
            .cartesian_product(0..BITS_PER_ROW)
            .filter(|&(i, j)| self.has_elf(i, j))
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
            .map(|row| [row.shift_east(), *row, row.shift_west()])
            .map_windows(|[above, cur, below]| Proposal::propose(above, cur, below, priority))
            .map_windows(|[above, cur, below]| Proposal::collide(above, cur, below))
            .enumerate()
            .for_each(
                |(i, Proposal([from_south, from_north, from_east, from_west]))| {
                    let destinations = from_north | from_south | from_west | from_east;
                    if destinations.is_empty() {
                        return;
                    }
                    moved = true;
                    grid.0[i + 1] &= !from_south;
                    grid.0[i - 1] &= !from_north;
                    grid.0[i] &= !from_west.shift_west();
                    grid.0[i] &= !from_east.shift_east();
                    grid.0[i] |= destinations;
                },
            );

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
    s.lines().enumerate().for_each(|(i, line)| {
        line.chars()
            .enumerate()
            .filter(|&(_, c)| c == '#')
            // Offsets are needed to give the elves enough space to expand out from their initial
            // positions.
            .for_each(|(j, _)| grid.insert(i + 24, j + 72))
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
        assert_eq!(task.grid.len(), 22);
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
