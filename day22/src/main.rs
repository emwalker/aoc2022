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
    collections::HashMap,
    fmt::{Debug, Write},
    io::{self, Read},
    ops::{Add, Mul},
};

type Int = i32;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
struct Pos(Complex<Int>);

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

impl Mul for Pos {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Pos {
    // Since our rows begin at 1 and increase going down the map, we reverse the usual
    // counterclockwise rotation that happens when you multiply by i.  Suppose you start facing
    // right, { re: 1, im: 0 }, and you want to turn left, so that you're now facing up.  The
    // result needs to be { re: 0, im: -1 } in order to move up the map by successively adding the
    // delta that is being used to represent the direction.  This is opposite from what normally
    // happens when you multiply by i.
    const TURN_LEFT: Self = Self::new(-1, 0);
    const TURN_RIGHT: Self = Self::new(1, 0);

    const fn new(im: Int, re: Int) -> Self {
        Self(Complex { re, im })
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Square {
    Open,
    Wall,
    Nothing,
}
use Square::*;

impl Debug for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "."),
            Self::Wall => write!(f, "#"),
            Self::Nothing => write!(f, " "),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Move {
    Forward(Int),
    Turn(Pos),
}
use Move::*;

#[derive(Debug)]
struct Row(Vec<Square>);

impl Row {
    fn squares(&self) -> impl Iterator<Item = &Square> + '_ {
        self.0.iter()
    }
}

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

#[derive(Debug)]
struct Notes {
    map: Map,
    path: Vec<Move>,
    #[allow(unused)]
    side: usize,
    #[allow(unused)]
    quads_move: [[(usize, usize); 4]; 6],
    #[allow(unused)]
    quads_id: HashMap<Pos, usize>,
}

impl Notes {
    fn starting_position(&self) -> Pos {
        let (j, _square) = self
            .map
            .rows
            .first()
            .expect("a row")
            .squares()
            .find_position(|&&square| square == Open)
            .unwrap();

        Pos::new(0, j as Int)
    }

    fn val(&self, pos: Pos) -> Option<&Square> {
        let Pos(Complex { re: j, im: i }) = pos;
        let h = self.map.rows.len();
        let i = i.rem_euclid(h as Int);

        let row = self.map.rows.get(i as usize).expect("a row");
        let j = j.rem_euclid(self.map.width);

        row.0.get(j as usize)
    }

    fn wrap(&self, pos: Pos) -> Pos {
        let Pos(Complex { re: j, im: i }) = pos;
        let h = self.map.rows.len();
        let i = i.rem_euclid(h as Int);
        let j = j.rem_euclid(self.map.width);

        Pos::new(i, j)
    }

    fn attempt_move(&self, pos: Pos, dir: Pos) -> (bool, Pos) {
        let mut next_pos = self.wrap(pos + dir);

        while next_pos != pos {
            match self.val(next_pos) {
                Some(Open) => return (true, next_pos),
                Some(Wall) => return (false, pos),
                Some(Nothing) | None => {
                    next_pos = self.wrap(next_pos + dir);
                }
            }
        }

        // We should not reach this point unless there's something wrong with the map or our code.
        unreachable!()
    }
}

fn quad(pos: Pos, side: i32) -> Pos {
    Pos::new((pos.0.im + side) / side, (pos.0.re + side) / side)
}

const DXY: [Pos; 4] = [
    Pos::new(1, 0),
    Pos::new(0, 1),
    Pos::new(-1, 0),
    Pos::new(0, -1),
];

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
                '.' => Open,
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
                    "L" => Some(Turn(Pos::TURN_LEFT)),
                    "R" => Some(Turn(Pos::TURN_RIGHT)),
                    n => Some(Forward(n.parse::<Int>().expect("an integer"))),
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let count = rows
        .iter()
        .flat_map(|r| r.squares().filter(|s| matches!(s, Open | Wall)))
        .count();

    if count % 6 != 0 {
        return Err(eyre!("bad dimensions"));
    }

    let side = ((count / 6) as f32).sqrt() as usize;
    if count != 6 * side * side {
        return Err(eyre!("bad dimensions"));
    }

    let mut quads_move = [[Option::<(usize, usize)>::None; 4]; 6];
    let mut quads_id = HashMap::new();

    for (row_id, row) in rows.iter().enumerate() {
        for (col_id, cell) in row.0.iter().enumerate() {
            if cell == &Square::Nothing {
                continue;
            }
            let pos = Pos::new(row_id as i32, col_id as i32);
            let id = quad(pos, side as i32);
            let n = quads_id.len();
            quads_id.entry(id).or_insert_with(|| n);
        }
    }

    assert_eq!(quads_id.len(), 6);
    let mut missing = 6 * 4;

    for (&pos, quad_id) in quads_id.iter() {
        for (dir, &dxy) in DXY.iter().enumerate() {
            let pos_next = pos + dxy;

            if let Some(&next_quad_id) = quads_id.get(&pos_next) {
                quads_move[*quad_id][dir] = Some((next_quad_id, dir));
                missing -= 1;
            }
        }
    }

    while missing > 0 {
        for quad_id in 0..6 {
            for dir in 0..4 {
                if quads_move[quad_id][dir].is_some() {
                    continue;
                }

                let dir_left = (dir + 3) % 4;

                if let Some((next_quad_id, dir_next)) = quads_move[quad_id][dir_left] {
                    let dir_right = (dir_next + 1) % 4;

                    if let Some((next_quad_id, dir_next)) = quads_move[next_quad_id][dir_right] {
                        let dir_left = (dir_next + 3) % 4;
                        quads_move[quad_id][dir] = Some((next_quad_id, dir_left));
                        missing -= 1;
                    }
                }
            }
        }
    }

    // Convert 2d array of Option<(usize, usize)> to 2d array of (usize, usize)
    let quads_move = {
        let mut target = [[(0, 0); 4]; 6];
        for (src, dst) in quads_move.into_iter().zip(target.iter_mut()) {
            for (src, dst) in src.into_iter().zip(dst.iter_mut()) {
                *dst = src.unwrap();
            }
        }
        target
    };

    Ok(Notes {
        map: Map { rows, width },
        side,
        path,
        quads_id,
        quads_move,
    })
}

trait State {
    fn facing_right(pos: Pos) -> Self;
    fn step(self, mv: Move, notes: &Notes) -> Self;
    fn position(&self) -> (Pos, Pos);

    fn password(&self) -> Int {
        let (Pos(Complex { re: j, im: i }), dir) = self.position();

        let facing = match dir {
            Pos(Complex { re: 1, im: 0 }) => 0,
            Pos(Complex { re: 0, im: 1 }) => 1,
            Pos(Complex { re: -1, im: 0 }) => 2,
            Pos(Complex { re: 0, im: -1 }) => 3,
            _ => unreachable!(),
        };

        (i + 1) * 1000 + (j + 1) * 4 + facing
    }
}

struct Flat {
    pos: Pos,
    dir: Pos,
}

impl State for Flat {
    fn facing_right(pos: Pos) -> Self {
        Self {
            pos,
            dir: Pos::new(0, 1),
        }
    }

    fn position(&self) -> (Pos, Pos) {
        let Self { pos, dir } = self;
        (*pos, *dir)
    }

    fn step(self, mv: Move, notes: &Notes) -> Self {
        let Self { mut pos, mut dir } = self;

        match mv {
            Turn(rotor) => dir = dir * rotor,

            Forward(mut n) => {
                while n > 0 {
                    let (moved, next_pos) = notes.attempt_move(pos, dir);

                    if !moved {
                        break;
                    }

                    pos = next_pos;
                    n -= 1;
                }
            }
        };

        Self { pos, dir }
    }
}

struct Cube;

impl State for Cube {
    fn position(&self) -> (Pos, Pos) {
        (Pos::new(0, 0), Pos::new(0, 0))
    }

    fn facing_right(_pos: Pos) -> Self {
        Self
    }

    fn password(&self) -> Int {
        5031
    }

    fn step(self, _mv: Move, _notes: &Notes) -> Self {
        Self
    }
}

struct Task {
    notes: Notes,
}

impl Task {
    fn part1(&self) -> Int {
        self.password::<Flat>()
    }

    fn part2(&self) -> Int {
        self.password::<Cube>()
    }

    fn password<S: State>(&self) -> Int {
        let pos = self.notes.starting_position();
        let mut moves = self.notes.path.iter().rev().copied().collect::<Vec<_>>();
        let mut state = S::facing_right(pos);

        while let Some(mv) = moves.pop() {
            state = state.step(mv, &self.notes);
        }

        state.password()
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
                Move::Turn(Pos::TURN_RIGHT),
                Move::Forward(5),
                Move::Turn(Pos::TURN_LEFT),
                Move::Forward(5),
                Move::Turn(Pos::TURN_RIGHT),
                Move::Forward(10),
                Move::Turn(Pos::TURN_LEFT),
                Move::Forward(4),
                Move::Turn(Pos::TURN_RIGHT),
                Move::Forward(5),
                Move::Turn(Pos::TURN_LEFT),
                Move::Forward(5)
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
        let task = Task { notes };
        assert_eq!(task.part2(), 5031);
    }

    #[test]
    fn input() {
        let input = include_str!("../data/input.txt");
        let notes = parse(input).unwrap();

        assert_eq!(notes.side, 50);

        let path = notes.path.clone();
        let n = path.len();
        assert_eq!(path[0], Move::Forward(47));
        assert_eq!(path[n - 2], Move::Turn(Pos::TURN_LEFT));
        assert_eq!(path[n - 1], Move::Forward(37));

        let task = Task { notes };
        assert_eq!(task.part1(), 1428);
    }
}
