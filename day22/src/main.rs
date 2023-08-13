// Reference solutions for part 2:
// - https://github.com/mfornet/advent-of-code-2022/blob/main/day22b/src/main.rs
//   Revisit.  Generic in dimensions.  Generic in shape.  Uses division by 6 and the square root of
//   the number of cells to find the length of a side.  Inneficient iteration over path commands.
// - https://github.com/idanarye/aoc-2022/blob/main/src/day22.rs
//   Revisit.  Ranges for row and column indexes.  Regex for parsing commands.  Index trait.
//   Generic in dimensions.  .
// - https://github.com/jchevertonwynne/advent-of-code-2022/blob/main/src/days/day22.rs
//   Use of a hash map with a fast hasher for the world map, not inserting anything into the map
//   for the blank regions.  Generic in dimensions (?).  Hard-coded in arrangement of faces to one
//   another (?).
// - https://github.com/Crazytieguy/advent-of-code/blob/master/2022/src/bin/day22/main.rs
//   Use of Rotation(Clockwise).  Use of Turn(Rotation).  Hard-coded in dimensions.  Hard coded in
//   shape.
// - https://github.com/sanraith/aoc2022/blob/aa33a4a7a8dfe6e522a5fe6af39c17b35892e465/aoc-lib/src/solutions/year2022/day22.rs
//   Has a context utility for printing out progress.  Generic solution.
// - https://github.com/HoshigaIkaro/aoc-2022/blob/main/src/days/day_22.rs
//   Use of integer assignments in enum.  Use of a template parameter to differentiate a "Flat"
//   implementation from a "Cube" implementation.  Hard-coded dimensions.
// - https://github.com/kelleyvanevert/adventofcode2022/blob/main/day22/src/main.rs
//   quaternion::{axis_angle, rotate_vector}, itertools::tuple_combinations.  Generic solution.
// - https://github.com/pavel1269/advent-of-code/blob/main/2022/src/day22/mod.rs
//   test_case to create permutations of a generic test
// - https://gist.github.com/mgedmin/71d632e40d4de5c9486a4616ffb53208
//   bit vectors to compute shared edges
//
use color_eyre::{eyre::eyre, Result};
use itertools::Itertools;
use num::complex::Complex;
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Write},
    io::{self, Read},
    ops::{Add, Mul, Sub},
};

type Int = i32;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
struct Pos(Complex<Int>);

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let Complex { im: i0, re: j0 } = self.0;
        let Complex { im: i1, re: j1 } = other.0;
        (i0, j0).cmp(&(i1, j1))
    }
}

impl Debug for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}i, {})", self.0.im, self.0.re)
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for Pos {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Pos {
    const fn new(i: Int, j: Int) -> Self {
        Self(Complex { im: i, re: j })
    }

    fn face(&self, side: Int) -> Face {
        let Complex { im: i, re: j } = self.0;
        let pos = Self::new((i + side) / side, (j + side) / side);
        Face(pos)
    }

    fn relative(&self, side: Int) -> Pos {
        let Complex { im: i, re: j } = self.0;
        Self::new(i.rem_euclid(side), j.rem_euclid(side))
    }

    //    00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15
    // 00                         .  .  .  #
    // 01                         .  #  .  .
    // 02                         #  .  .  .
    // 03                         .  .  .  .
    // 04 .  .  .  #  .  .  E  .  .  .  .  #
    // 05 .  .  .  .  .  .  .  .  #  .  .  A
    // 06 .  .  #  .  .  .  .  #  .  .  .  .
    // 07 .  D  .  .  .  .  .  .  .  .  #  .
    // 08                         .  .  .  #  .  .  B  .
    // 09                         .  .  .  .  .  #  .  .
    // 10                         .  #  .  .  .  .  .  .
    // 11                         .  .  C  .  .  .  #  .
    //
    // - At A, and move to the right -> B, facing down
    //
    //   top left corner: (8, 12)
    //   (4, 11)  -> (8, 15)  [4, 4]
    //   {0, 3}      {0, 3}   [0, side-i-1]
    //
    //   (7, 11)  -> (8, 12)  [1, 1]
    //   {3, 3}      {0, 0}   [0, side-i-1]
    //
    fn rotate(&self, old: Dxy, new: Dxy, side: Int) -> Self {
        let Complex { im: i, re: j } = self.0;

        match (old, new) {
            (Dxy::DOWN, Dxy::UP) => Self::new(side - 1, side - j - 1),
            (Dxy::RIGHT, Dxy::DOWN) => Self::new(0, side - i - 1),
            (Dxy::UP, Dxy::RIGHT) => Self::new(j, 0),
            (Dxy::UP, Dxy::UP) => Self::new(side - 1, j),
            (Dxy::DOWN, Dxy::DOWN) => Self::new(0, j),
            (Dxy::DOWN, Dxy::LEFT) => Self::new(j, side - 1),
            (Dxy::LEFT, Dxy::DOWN) => Self::new(0, i),
            (Dxy::LEFT, Dxy::RIGHT) => Self::new(side - i - 1, 0),
            (Dxy::RIGHT, Dxy::LEFT) => Self::new(side - i - 1, side - 1),
            (Dxy::RIGHT, Dxy::UP) => Self::new(side - 1, i),
            _ => panic!("{:?} -> {:?}", old, new),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
struct Dxy(Pos);

impl Dxy {
    const UP: Dxy = Dxy::new(-1, 0);
    const RIGHT: Dxy = Dxy::new(0, 1);
    const DOWN: Dxy = Dxy::new(1, 0);
    const LEFT: Dxy = Dxy::new(0, -1);
    const DIRECTIONS: [Dxy; 4] = [Self::DOWN, Self::RIGHT, Self::UP, Self::LEFT];

    const fn new(im: Int, re: Int) -> Self {
        Self(Pos::new(im, re))
    }

    fn idx(&self) -> Int {
        match *self {
            Self::RIGHT => 0,
            Self::DOWN => 1,
            Self::LEFT => 2,
            Self::UP => 3,
            _ => panic!("not a direction"),
        }
    }

    fn turn_left(&self) -> Self {
        Self(self.0 * Pos(Dir::COUNTERCLOCKWISE.0))
    }

    fn turn_right(&self) -> Self {
        Self(self.0 * Pos(Dir::CLOCKWISE.0))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Dir(Complex<Int>);

impl Dir {
    // Since our rows begin at 1 and increase going down the map, we reverse the usual
    // counterclockwise rotation that happens when you multiply by i.  Suppose you start facing
    // right, { re: 1, im: 0 }, and you want to turn left, so that you're now facing up.  The
    // result needs to be { re: 0, im: -1 } in order to move up the map by successively adding the
    // delta that is being used to represent the direction.  This is opposite from what normally
    // happens when you multiply by i.
    const COUNTERCLOCKWISE: Self = Self::new(-1, 0);
    const CLOCKWISE: Self = Self::new(1, 0);

    const fn new(im: Int, re: Int) -> Self {
        Self(Complex { im, re })
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
struct Face(Pos);

impl Face {
    #[allow(unused)]
    fn new(i: Int, j: Int) -> Self {
        Self(Pos::new(i, j))
    }

    fn top_left_corner(&self, side: Int) -> Pos {
        let Pos(Complex { im: i, re: j }) = self.0;
        Pos::new((i - 1) * side, (j - 1) * side)
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Square {
    Tile,
    Wall,
    Nothing,
}
use Square::*;

impl Debug for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tile => write!(f, "."),
            Self::Wall => write!(f, "#"),
            Self::Nothing => write!(f, " "),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Move {
    Forward(Int),
    Rotate(Dir),
}
use Move::*;

#[derive(Clone, Debug)]
struct Row(Vec<Square>);

impl Row {
    fn squares(&self) -> impl Iterator<Item = &Square> + '_ {
        self.0.iter()
    }
}

#[derive(Clone)]
struct Map {
    rows: Vec<Row>,
    width: Int,
}

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("\n\n")?;

        for row in &self.rows {
            for s in &row.0 {
                write!(f, "{:?}", s)?;
            }
            f.write_char('\n')?;
        }

        f.write_char('\n')
    }
}

#[derive(Clone, Debug)]
struct Notes {
    map: Map,
    path: Vec<Move>,
    side: Int,
    transitions: HashMap<(Face, Dxy), (Face, Dxy)>,
}

impl Notes {
    fn starting_position(&self) -> Pos {
        let (j, _square) = self
            .map
            .rows
            .first()
            .expect("a row")
            .squares()
            .find_position(|&&square| square == Tile)
            .unwrap();

        Pos::new(0, j as Int)
    }

    fn val(&self, pos: Pos) -> Option<&Square> {
        let Pos(Complex { im: i, re: j }) = pos;
        match self.map.rows.get(i as usize) {
            Some(row) => row.0.get(j as usize),
            _ => None,
        }
    }
}

fn parse(s: &str) -> Result<Notes> {
    let mut rows = vec![];
    let mut lines = s.lines();
    let mut width = 0;

    for line in &mut lines {
        if line.is_empty() {
            break;
        }

        let mut row = vec![];
        width = width.max(line.len() as Int);

        for c in line.chars() {
            let square = match c {
                ' ' => Nothing,
                '.' => Tile,
                '#' => Wall,
                _ => return Err(eyre!("unknown square: {}", c)),
            };

            row.push(square);
        }

        rows.push(Row(row));
    }

    if rows.is_empty() {
        return Err(eyre!("map cannot be empty"));
    }

    let commands = lines.next().unwrap();
    let pattern = Regex::new(r"\d+|[LR]").unwrap();
    let path = pattern
        .captures_iter(commands)
        .flat_map(|cap| {
            if let Some(s) = cap.get(0) {
                match s.as_str() {
                    "L" => Some(Rotate(Dir::COUNTERCLOCKWISE)),
                    "R" => Some(Rotate(Dir::CLOCKWISE)),
                    n => Some(Forward(n.parse::<Int>().expect("an integer"))),
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let count = rows
        .iter()
        .flat_map(|r| r.squares().filter(|s| matches!(s, Tile | Wall)))
        .count();

    if count % 6 != 0 {
        return Err(eyre!("bad dimensions"));
    }

    let side = ((count / 6) as f32).sqrt() as Int;
    if count as Int != 6 * side * side {
        return Err(eyre!("bad dimensions"));
    }

    let mut transitions = HashMap::new();
    let mut faces = HashSet::new();

    for (i, row) in rows.iter().enumerate() {
        for (j, cell) in row.0.iter().enumerate() {
            if cell == &Square::Nothing {
                continue;
            }
            let face = Pos::new(i as i32, j as i32).face(side);
            faces.insert(face);
        }
    }

    assert_eq!(faces.len(), 6);
    let mut missing = 6 * 4;

    for &face in &faces {
        for dxy in Dxy::DIRECTIONS {
            let next_face = Face(face.0 + dxy.0);

            if let Some(&next_face) = faces.get(&next_face) {
                transitions.entry((face, dxy)).or_insert((next_face, dxy));
                missing -= 1;
            }
        }
    }

    while missing > 0 {
        for &face in &faces {
            for dxy in Dxy::DIRECTIONS {
                if transitions.get(&(face, dxy)).is_some() {
                    continue;
                }

                let move_left = dxy.turn_left();

                if let Some(&(next_face, next_dxy)) = transitions.get(&(face, move_left)) {
                    let move_right = next_dxy.turn_right();

                    if let Some(&(next_face, next_dxy)) = transitions.get(&(next_face, move_right))
                    {
                        let move_left = next_dxy.turn_left();
                        transitions
                            .entry((face, dxy))
                            .or_insert((next_face, move_left));
                        missing -= 1;
                    }
                }
            }
        }
    }

    Ok(Notes {
        map: Map { rows, width },
        side,
        path,
        transitions,
    })
}

trait State {
    fn advance(&mut self) -> &mut Self;
    fn position(&self) -> (Pos, Dxy);

    fn password(&self) -> Int {
        let (Pos(Complex { re: j, im: i }), dxy) = self.position();
        (i + 1) * 1000 + (j + 1) * 4 + dxy.idx()
    }
}

struct Flat {
    pos: Pos,
    dxy: Dxy,
    notes: Notes,
}

impl State for Flat {
    fn position(&self) -> (Pos, Dxy) {
        let Self { pos, dxy, .. } = self;
        (*pos, *dxy)
    }

    fn advance(&mut self) -> &mut Self {
        let mut moves = self.notes.path.iter().rev().copied().collect::<Vec<_>>();

        while let Some(mv) = moves.pop() {
            self.step(mv);
        }

        self
    }
}

impl Flat {
    fn new(notes: Notes) -> Self {
        Self {
            pos: notes.starting_position(),
            dxy: Dxy::RIGHT,
            notes,
        }
    }

    fn wrap(&self, pos: Pos) -> Pos {
        let Pos(Complex { re: j, im: i }) = pos;
        let h = self.notes.map.rows.len();
        let i = i.rem_euclid(h as Int);
        let j = j.rem_euclid(self.notes.map.width);

        Pos::new(i, j)
    }

    fn attempt_move(&self, pos: Pos, dxy: Dxy) -> (bool, Pos) {
        let mut next_pos = self.wrap(pos + dxy.0);

        while next_pos != pos {
            match self.notes.val(next_pos) {
                Some(Tile) => return (true, next_pos),
                Some(Wall) => return (false, pos),
                Some(Nothing) | None => {
                    next_pos = self.wrap(next_pos + dxy.0);
                }
            }
        }

        // We should not reach this point unless there's something wrong with the map or our code.
        unreachable!()
    }

    fn step(&mut self, mv: Move) {
        match mv {
            Rotate(rotor) => self.dxy = Dxy(self.dxy.0 * Pos(rotor.0)),

            Forward(mut n) => {
                while n > 0 {
                    let (moved, next_pos) = self.attempt_move(self.pos, self.dxy);

                    if !moved {
                        break;
                    }

                    self.pos = next_pos;
                    n -= 1;
                }
            }
        };
    }
}

struct Cube {
    pos: Pos,
    dxy: Dxy,
    notes: Notes,
}

impl State for Cube {
    fn position(&self) -> (Pos, Dxy) {
        (self.pos, self.dxy)
    }

    fn advance(&mut self) -> &mut Self {
        let mut moves = self.notes.path.iter().rev().copied().collect::<Vec<_>>();

        while let Some(mv) = moves.pop() {
            self.step(mv);
        }

        self
    }
}

impl Cube {
    fn new(notes: Notes) -> Self {
        Self {
            pos: notes.starting_position(),
            dxy: Dxy::RIGHT,
            notes,
        }
    }

    #[allow(unused)]
    fn face(&self) -> Face {
        self.pos.face(self.notes.side)
    }

    fn step(&mut self, mv: Move) {
        match mv {
            Rotate(rotor) => self.dxy = Dxy(self.dxy.0 * Pos(rotor.0)),

            Forward(mut n) => {
                while n > 0 {
                    let (moved, next_pos, next_dxy) = self.attempt_move(self.pos, self.dxy, 0);
                    n -= 1;

                    if !moved {
                        break;
                    }

                    self.dxy = next_dxy;
                    self.pos = next_pos;
                }
            }
        };
    }

    fn attempt_move(&self, pos: Pos, dxy: Dxy, depth: i8) -> (bool, Pos, Dxy) {
        let side = self.notes.side;
        let next_pos = pos + dxy.0;

        match self.notes.val(next_pos) {
            Some(Tile) => (true, next_pos, dxy),
            Some(Wall) => (false, pos, dxy),

            Some(Nothing) | None => {
                if depth > 1 {
                    // If we reach this point, we did not stitch together the faces of the cube
                    // correctly from the puzzle input, or the puzzle input is malformed.
                    unreachable!();
                }

                let face = pos.face(side);
                let &(next_face, next_dxy) = self
                    .notes
                    .transitions
                    .get(&(face, dxy))
                    .expect("a transition");

                let corner = next_face.top_left_corner(side);
                let next_pos = corner + pos.relative(side).rotate(dxy, next_dxy, side);
                self.attempt_move(next_pos - next_dxy.0, next_dxy, depth + 1)
            }
        }
    }
}

struct Task {
    notes: Notes,
}

impl Task {
    fn part1(&self) -> Int {
        Flat::new(self.notes.clone()).advance().password()
    }

    fn part2(&self) -> Int {
        Cube::new(self.notes.clone()).advance().password()
    }
}

fn main() -> Result<()> {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;

    let notes = parse(&s)?;
    let task = Task { notes };
    println!("part 1: password: {}", task.part1());
    println!("part 2: password: {}", task.part2());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let input = include_str!("../data/example.txt");
        let notes = parse(input).unwrap();

        assert_eq!(notes.side, 4);

        assert_eq!(notes.map.rows.last().unwrap().0.len(), 16);
        assert_eq!(
            notes.path,
            &[
                Move::Forward(10),
                Move::Rotate(Dir::CLOCKWISE),
                Move::Forward(5),
                Move::Rotate(Dir::COUNTERCLOCKWISE),
                Move::Forward(5),
                Move::Rotate(Dir::CLOCKWISE),
                Move::Forward(10),
                Move::Rotate(Dir::COUNTERCLOCKWISE),
                Move::Forward(4),
                Move::Rotate(Dir::CLOCKWISE),
                Move::Forward(5),
                Move::Rotate(Dir::COUNTERCLOCKWISE),
                Move::Forward(5)
            ]
        );
    }

    #[test]
    fn corners() {
        let input = include_str!("../data/example.txt");
        let notes = parse(input).unwrap();
        let side = notes.side;

        let faces = notes
            .transitions
            .keys()
            .map(|(face, _)| face)
            .collect::<HashSet<_>>();

        let corners = faces
            .iter()
            .map(|face| face.top_left_corner(side))
            .sorted()
            .collect::<Vec<_>>();

        assert_eq!(
            &corners,
            &[
                Pos::new(0, 8),
                Pos::new(4, 0),
                Pos::new(4, 4),
                Pos::new(4, 8),
                Pos::new(8, 8),
                Pos::new(8, 12)
            ]
        );
    }

    #[test]
    fn part1() {
        let input = include_str!("../data/example.txt");
        let notes = parse(input).unwrap();
        let task = Task { notes };
        assert_eq!(task.part1(), 6032);
    }

    #[test]
    fn part2() {
        let input = include_str!("../data/example.txt");
        let notes = parse(input).unwrap();
        let mut moves = notes.path.iter().rev().copied().collect::<Vec<_>>();
        let mut state = Cube::new(notes);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(1, 3));
        assert_eq!(state.pos, Pos::new(0, 10));
        assert_eq!(state.dxy, Dxy::RIGHT);
        assert_eq!(state.password(), 1044);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(1, 3));
        assert_eq!(state.pos, Pos::new(0, 10));
        assert_eq!(state.dxy, Dxy::DOWN);
        assert_eq!(state.password(), 1045);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(2, 3));
        assert_eq!(state.pos, Pos::new(5, 10));
        assert_eq!(state.dxy, Dxy::DOWN);
        assert_eq!(state.password(), 6045);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(2, 3));
        assert_eq!(state.pos, Pos::new(5, 10));
        assert_eq!(state.dxy, Dxy::RIGHT);
        assert_eq!(state.password(), 6044);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(3, 4));
        assert_eq!(state.pos, Pos::new(10, 14));
        assert_eq!(state.dxy, Dxy::DOWN);
        assert_eq!(state.password(), 11061);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(3, 4));
        assert_eq!(state.pos, Pos::new(10, 14));
        assert_eq!(state.dxy, Dxy::LEFT);
        assert_eq!(state.password(), 11062);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(3, 3));
        assert_eq!(state.pos, Pos::new(10, 10));
        assert_eq!(state.dxy, Dxy::LEFT);
        assert_eq!(state.password(), 11046);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(3, 3));
        assert_eq!(state.pos, Pos::new(10, 10));
        assert_eq!(state.dxy, Dxy::DOWN);
        assert_eq!(state.password(), 11045);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(2, 1));
        assert_eq!(state.pos, Pos::new(5, 1));
        assert_eq!(state.dxy, Dxy::UP);
        assert_eq!(state.password(), 6011);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(2, 1));
        assert_eq!(state.pos, Pos::new(5, 1));
        assert_eq!(state.dxy, Dxy::RIGHT);
        assert_eq!(state.password(), 6008);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(2, 2));
        assert_eq!(state.pos, Pos::new(5, 6));
        assert_eq!(state.dxy, Dxy::RIGHT);
        assert_eq!(state.password(), 6028);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(2, 2));
        assert_eq!(state.pos, Pos::new(5, 6));
        assert_eq!(state.dxy, Dxy::UP);
        assert_eq!(state.password(), 6031);

        state.step(moves.pop().unwrap());
        assert_eq!(state.face(), Face::new(2, 2));
        assert_eq!(state.pos, Pos::new(4, 6));
        assert_eq!(state.dxy, Dxy::UP);
        assert_eq!(state.password(), 5031);

        assert!(moves.is_empty());
    }

    #[test]
    fn input() {
        let input = include_str!("../data/input.txt");
        let notes = parse(input).unwrap();

        assert_eq!(notes.side, 50);

        let path = notes.path.clone();
        let n = path.len();
        assert_eq!(path[0], Move::Forward(47));
        assert_eq!(path[n - 2], Move::Rotate(Dir::COUNTERCLOCKWISE));
        assert_eq!(path[n - 1], Move::Forward(37));

        let task = Task { notes };
        assert_eq!(task.part1(), 1428);

        let p2 = task.part2();
        assert!(p2 > 41381);
        assert!(p2 > 124158);
        assert_eq!(p2, 142380);
    }
}
