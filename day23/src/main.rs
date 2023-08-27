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
use fxhash::{FxHashSet, FxHasher};
use std::{
    fmt::{Debug, Write},
    hash::BuildHasherDefault,
    io::{self, Read},
    ops::Add,
};

type Int = i32;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Pos {
    i: Int,
    j: Int,
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            i: self.i + rhs.i,
            j: self.j + rhs.j,
        }
    }
}

impl Pos {
    #[allow(unused)]
    fn new(i: Int, j: Int) -> Self {
        Self { i, j }
    }

    #[allow(unused)]
    fn coords(&self) -> (Int, Int) {
        (self.i, self.j)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Dij(Pos);

impl Dij {
    const N: Self = Self::new(-1, 0);
    const NE: Self = Self::new(-1, 1);
    const E: Self = Self::new(0, 1);
    const SE: Self = Self::new(1, 1);
    const S: Self = Self::new(1, 0);
    const SW: Self = Self::new(1, -1);
    const W: Self = Self::new(0, -1);
    const NW: Self = Self::new(-1, -1);

    const DIRECTIONS: [Self; 8] = [
        Self::N,
        Self::NE,
        Self::E,
        Self::SE,
        Self::S,
        Self::SW,
        Self::W,
        Self::NW,
    ];

    const fn new(i: Int, j: Int) -> Self {
        Self(Pos { i, j })
    }
}

struct DijGroup(Dij);

impl DijGroup {
    const DIRECTIONS: [Self; 4] = [Self(Dij::N), Self(Dij::S), Self(Dij::W), Self(Dij::E)];

    fn nearby(&self) -> &[Dij; 3] {
        match self.0 {
            Dij::N => &[Dij::N, Dij::NE, Dij::NW],
            Dij::S => &[Dij::S, Dij::SE, Dij::SW],
            Dij::E => &[Dij::E, Dij::NE, Dij::SE],
            Dij::W => &[Dij::W, Dij::NW, Dij::SW],
            _ => unreachable!(),
        }
    }

    fn pos(&self) -> Pos {
        self.0 .0
    }
}

#[derive(Clone)]
struct Map(FxHashSet<Pos>);

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ((max_i, min_i), (max_j, min_j)) = self.bounds();

        f.write_char('\n')?;

        for i in min_i..=max_i {
            for j in min_j..=max_j {
                let pos = Pos::new(i, j);
                if self.0.contains(&pos) {
                    f.write_char('#')?;
                } else {
                    f.write_char('.')?;
                }
            }
            f.write_char('\n')?;
        }

        Ok(())
    }
}

impl Map {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn bounds(&self) -> ((Int, Int), (Int, Int)) {
        let (mut min_i, mut max_i) = (Int::MAX, Int::MIN);
        let (mut min_j, mut max_j) = (Int::MAX, Int::MIN);

        for pos in self.iter() {
            let &Pos { i, j } = pos;
            min_i = min_i.min(i);
            max_i = max_i.max(i);
            min_j = min_j.min(j);
            max_j = max_j.max(j);
        }

        ((max_i, min_i), (max_j, min_j))
    }

    fn dimensions(&self) -> (Int, Int) {
        let ((max_i, min_i), (max_j, min_j)) = self.bounds();
        (max_i - min_i + 1, max_j - min_j + 1)
    }

    fn iter(&self) -> impl Iterator<Item = &Pos> {
        self.0.iter()
    }

    fn contains(&self, pos: &Pos) -> bool {
        self.0.contains(pos)
    }

    fn destination(&self, pos: Pos, round: usize) -> Pos {
        let adjacent = Dij::DIRECTIONS
            .iter()
            .any(|dij| self.contains(&(pos + dij.0)));

        if adjacent {
            for idx in round..round + 4 {
                let dir = &DijGroup::DIRECTIONS[idx % 4];
                if !dir.nearby().iter().any(|dij| self.contains(&(pos + dij.0))) {
                    return dir.pos() + pos;
                }
            }
        }

        pos
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Proposal {
    from: Pos,
    to: Pos,
}

impl Proposal {
    #[allow(unused)]
    fn coords(&self) -> [(Int, Int); 2] {
        [self.from.coords(), self.to.coords()]
    }
}

struct State {
    map: Map,
    round: usize,
    moved: bool,
    elf_count: usize,
}

impl State {
    fn empty_tiles(&self) -> Int {
        let elf_count = self.map.len() as Int;
        let (height, width) = self.map.dimensions();
        assert!(height * width >= elf_count);
        height * width - elf_count
    }

    fn step(self) -> Self {
        let Self {
            round,
            map,
            elf_count,
            ..
        } = self;

        let hasher: BuildHasherDefault<FxHasher> = BuildHasherDefault::default();
        let mut next_map = FxHashSet::with_capacity_and_hasher(map.len(), hasher);
        let mut num_moves = 0;

        for &elf in map.iter() {
            let next_pos = map.destination(elf, self.round);

            if elf == next_pos {
                next_map.insert(next_pos);
            } else if !next_map.insert(next_pos) {
                // Something else attempted to move there; let's back out the change
                next_map.remove(&next_pos);
                next_map.insert(elf);
                next_map.insert(Pos::new(next_pos.i * 2 - elf.i, next_pos.j * 2 - elf.j));
                num_moves -= 2;
            } else {
                num_moves += 1;
            }
        }

        debug_assert_eq!(elf_count, next_map.len());

        Self {
            map: Map(next_map),
            round: round + 1,
            elf_count,
            moved: num_moves != 0,
        }
    }
}

struct Task {
    map: Map,
}

impl Task {
    fn part1(&self) -> Int {
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
        let map = self.map.clone();
        let elf_count = map.len();
        debug_assert!(elf_count > 0, "expected at least one elf");

        State {
            elf_count,
            round: 0,
            map,
            moved: true,
        }
    }
}

fn parse(s: &str) -> Result<Task> {
    let map = s
        .trim()
        .lines()
        .enumerate()
        .flat_map(|(i, l)| {
            l.trim().chars().enumerate().filter_map(move |(j, c)| {
                if c == '#' {
                    Some(Pos {
                        i: i as Int,
                        j: j as Int,
                    })
                } else {
                    None
                }
            })
        })
        .collect::<FxHashSet<Pos>>();

    let map = Map(map);
    Ok(Task { map })
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
        assert_eq!(task.map.len(), 22);
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
            &format!("{:?}", state.map),
            "##
             #.
             ..
             ##",
        );

        state = step(state, 1);

        assert_same(
            &format!("{:?}", state.map),
            "##
             ..
             #.
             .#
             #.",
        );

        state = step(state, 1);

        assert_same(
            &format!("{:?}", state.map),
            ".##.
             #...
             ...#
             ....
             .#..",
        );

        state = step(state, 1);

        assert_same(
            &format!("{:?}", state.map),
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
            &format!("{:?}", state.map),
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
            &format!("{:?}", state.map),
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
            &format!("{:?}", state.map),
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
            &format!("{:?}", state.map),
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
            &format!("{:?}", state.map),
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
            &format!("{:?}", state.map),
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
            &format!("{:?}", state.map),
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

        let (height, width) = state.map.dimensions();
        assert_eq!(12, width);
        assert_eq!(11, height);
        assert_eq!(state.empty_tiles(), 110);

        let state = step(state, 9);
        assert_eq!(state.round, 19);

        // Round 20
        assert_same(
            &format!("{:?}", state.map),
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
