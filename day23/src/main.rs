use color_eyre::Result;
use counter::Counter;
use std::{
    collections::{BTreeSet, VecDeque},
    fmt::{Debug, Write},
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

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.i, self.j).cmp(&(other.i, other.j))
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
struct Map(BTreeSet<Pos>);

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

    fn clear_in_direction(&self, pos: Pos, dir: &DijGroup) -> bool {
        dir.nearby().iter().all(|dxy| {
            let neighor = pos + dxy.0;
            !self.contains(&neighor)
        })
    }

    fn clear_around(&self, pos: Pos) -> bool {
        Dij::DIRECTIONS.iter().all(|dxy| {
            let neighbor = pos + dxy.0;
            !self.contains(&neighbor)
        })
    }

    fn remove(&mut self, pos: &Pos) {
        self.0.remove(pos);
    }

    fn insert(&mut self, pos: &Pos) {
        self.0.insert(*pos);
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

#[derive(Debug, Eq, PartialEq)]
struct Proposals(Vec<Proposal>);

impl Proposals {
    fn can_move(&self) -> bool {
        !self.0.is_empty()
    }

    #[allow(unused)]
    fn len(&self) -> usize {
        self.0.len()
    }

    fn iter(&self) -> impl Iterator<Item = &Proposal> {
        self.0.iter()
    }
}

struct State {
    map: Map,
    round: Int,
    elf_count: usize,
    directions: VecDeque<DijGroup>,
    proposals: Option<Proposals>,
}

impl State {
    fn can_move(&mut self) -> bool {
        if let Some(proposals) = &self.proposals {
            return proposals.can_move();
        }

        let mut sought: Counter<Pos> = Counter::new();
        let mut possible = vec![];

        for &from in self.map.iter() {
            if self.map.clear_around(from) {
                // No need to spread further out
                sought[&from] += 1;
                continue;
            }

            for dir in &self.directions {
                if !self.map.clear_in_direction(from, dir) {
                    continue;
                }

                let to = from + dir.pos();
                sought[&to] += 1;
                possible.push(Proposal { from, to });
                break;
            }
        }

        let proposals = Proposals(
            possible
                .into_iter()
                .filter(|proposal| sought[&proposal.to] < 2)
                .collect::<Vec<_>>(),
        );

        let res = proposals.can_move();
        std::mem::swap(&mut self.proposals, &mut Some(proposals));
        res
    }

    fn empty_tiles(&self) -> Int {
        let elf_count = self.map.len() as Int;
        let (height, width) = self.map.dimensions();
        assert!(height * width >= elf_count);
        height * width - elf_count
    }

    fn step(self) -> Self {
        let Self {
            proposals,
            round,
            mut map,
            mut directions,
            ..
        } = self;

        for Proposal { from, to } in proposals.expect("can_move() called").iter() {
            map.remove(from);
            map.insert(to);
        }

        let elf_count = map.len();
        debug_assert_eq!(self.elf_count, elf_count, "unexpected number of elves");

        directions.rotate_left(1);

        Self {
            map,
            round: round + 1,
            elf_count,
            directions,
            proposals: None,
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

    fn part2(&self) -> Int {
        self.advance(100_000).round + 1
    }

    fn advance(&self, rounds: Int) -> State {
        let mut state = self.start();

        while state.can_move() && state.round < rounds {
            state = state.step()
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
            proposals: None,
            directions: VecDeque::from([
                DijGroup(Dij::N),
                DijGroup(Dij::S),
                DijGroup(Dij::W),
                DijGroup(Dij::E),
            ]),
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
        .collect::<BTreeSet<Pos>>();

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

    #[test]
    fn parsing() {
        let task = parse(example()).unwrap();
        assert_eq!(task.map.len(), 22);
    }

    #[test]
    fn can_move() {
        let task = parse(example()).unwrap();
        let mut state = task.start();
        assert!(state.can_move());
        state = state.step();
        assert!(state.can_move());
    }

    #[test]
    fn empty_tiles() {
        let task = parse(example()).unwrap();
        let mut state = task.start();

        assert!(state.can_move());
        assert_eq!(state.empty_tiles(), 27);

        state = state.step();
        assert!(state.can_move());
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

        fn coords(proposals: &Option<Proposals>) -> Vec<[(Int, Int); 2]> {
            proposals
                .as_ref()
                .unwrap()
                .iter()
                .map(|proposal| proposal.coords())
                .collect()
        }

        assert!(state.can_move());
        assert_eq!(
            coords(&state.proposals),
            vec![[(1, 2), (0, 2)], [(1, 3), (0, 3)], [(4, 3), (3, 3)]],
        );

        state = state.step();
        assert!(state.can_move());
        assert_eq!(
            coords(&state.proposals),
            vec![
                [(0, 2), (1, 2)],
                [(0, 3), (1, 3)],
                [(2, 2), (2, 1)],
                [(3, 3), (3, 4)],
                [(4, 2), (5, 2)]
            ],
        );

        state = state.step();
        assert!(state.can_move());
        assert_eq!(
            coords(&state.proposals),
            vec![[(1, 2), (0, 2)], [(1, 3), (1, 4)], [(2, 1), (2, 0)]],
        );

        state = state.step();
        assert!(!state.can_move());

        let coords = state.map.iter().map(|pos| pos.coords()).collect::<Vec<_>>();
        assert_eq!(coords, vec![(0, 2), (1, 4), (2, 0), (3, 4), (5, 2)]);
    }

    #[test]
    fn part1() {
        let task = parse(example()).unwrap();
        let mut state = task.start();

        fn step(mut state: State, steps: usize) -> State {
            for _ in 0..steps {
                assert!(state.can_move());
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

        let mut state = step(state, 9);
        assert!(!state.can_move());
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
