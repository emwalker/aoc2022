use color_eyre::{eyre::eyre, Result};
use num::complex::Complex;
use std::{
    fmt::{Debug, Write},
    io::{self, Read},
    ops::{Add, Mul},
};

type Int = i32;

#[derive(Copy, Clone, Eq, PartialEq)]
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

impl Pos {
    const fn new(im: Int, re: Int) -> Self {
        Self(Complex { re, im })
    }
}

impl Mul for Pos {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Square {
    Open,
    Wall,
    OffBoard,
}

impl Debug for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "."),
            Self::Wall => write!(f, "#"),
            Self::OffBoard => write!(f, " "),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Move {
    Forward(Int),
    TurnLeft,
    TurnRight,
}

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
}

impl Notes {
    fn starting_position(&self) -> Pos {
        let first = self.map.rows.first().expect("a row");

        for (j, &square) in first.squares().enumerate() {
            if square == Square::Open {
                return Pos::new(0, j as Int);
            }
        }

        unreachable!()
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
                Some(Square::Open) => return (true, next_pos),
                Some(Square::Wall) => return (false, pos),
                Some(Square::OffBoard) | None => {
                    next_pos = self.wrap(next_pos + dir);
                }
            }
        }

        // We should not reach this point unless there's something wrong with the map or our code.
        unreachable!()
    }
}

struct NumBuilder {
    chars: Vec<char>,
}

impl NumBuilder {
    fn new() -> Self {
        Self { chars: vec![] }
    }

    fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    fn build(&mut self) -> Int {
        let n = self
            .chars
            .iter()
            .collect::<String>()
            .parse::<Int>()
            .expect("failed to parse int");
        self.chars = vec![];
        n
    }

    fn push(&mut self, c: char) {
        self.chars.push(c)
    }
}

fn parse(s: &str) -> Result<Notes> {
    let mut rows = vec![];
    let lines = s.lines().collect::<Vec<&str>>();
    let mut width = 0;

    for line in lines.iter() {
        if line.is_empty() {
            break;
        }

        let mut row = vec![];
        width = width.max(line.len() as Int);

        for c in line.chars() {
            let square = match c {
                ' ' => Square::OffBoard,
                '.' => Square::Open,
                '#' => Square::Wall,
                _ => return Err(eyre!("unknown square: {}", c)),
            };

            row.push(square);
        }

        rows.push(Row(row));
    }

    if rows.is_empty() {
        return Err(eyre!("map cannot be empty"));
    }

    let s = lines
        .last()
        .ok_or(eyre!("expected a path"))?
        .chars()
        .collect::<Vec<char>>();
    let mut path = vec![];
    let mut curr_num = NumBuilder::new();

    for c in s {
        match c {
            'L' => {
                if !curr_num.is_empty() {
                    path.push(Move::Forward(curr_num.build()));
                }

                path.push(Move::TurnLeft)
            }

            'R' => {
                if !curr_num.is_empty() {
                    path.push(Move::Forward(curr_num.build()));
                }

                path.push(Move::TurnRight)
            }

            _ => curr_num.push(c),
        }
    }

    if !curr_num.is_empty() {
        path.push(Move::Forward(curr_num.build()));
    }

    Ok(Notes {
        map: Map { rows, width },
        path,
    })
}

#[derive(Debug)]
struct State {
    pos: Pos,
    dir: Pos,
}

impl State {
    fn facing_right(pos: Pos) -> Self {
        Self {
            pos,
            dir: Pos::new(0, 1),
        }
    }
}

impl State {
    // Since our rows begin at 1 and increase going down the map, we reverse the usual
    // counterclockwise rotation that happens when you multiply by i.  Suppose you start facing
    // right, { re: 1, im: 0}, and you want to turn left, so that you're now facing up.  The
    // result needs to be { re: 0, im: -1 } in order to move up the map by successively adding the
    // delta that is being used to represent the direction.  This is opposite from what normally
    // happens when you multiply by i.
    const TURN_LEFT: Pos = Pos::new(-1, 0);
    const TURN_RIGHT: Pos = Pos::new(1, 0);

    fn password(&self) -> Int {
        let Self {
            pos: Pos(Complex { re: j, im: i }),
            dir,
        } = self;

        let facing = match dir {
            Pos(Complex { re: 1, im: 0 }) => 0,
            Pos(Complex { re: 0, im: 1 }) => 1,
            Pos(Complex { re: -1, im: 0 }) => 2,
            Pos(Complex { re: 0, im: -1 }) => 3,
            _ => unreachable!(),
        };

        (i + 1) * 1000 + (j + 1) * 4 + facing
    }

    fn step(self, mv: Move, notes: &Notes) -> Self {
        let Self { mut pos, mut dir } = self;

        match mv {
            Move::TurnLeft => dir = dir * Self::TURN_LEFT,
            Move::TurnRight => dir = dir * Self::TURN_RIGHT,

            Move::Forward(mut n) => {
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

struct Task {
    notes: Notes,
}

impl Task {
    fn password(&self) -> Int {
        let pos = self.notes.starting_position();
        let mut moves = self.notes.path.iter().rev().copied().collect::<Vec<_>>();
        let mut state = State::facing_right(pos);

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
    println!("part 1: password: {}", task.password());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let input = include_str!("../data/example.txt");
        let notes = parse(input).unwrap();

        assert_eq!(notes.map.rows.last().unwrap().0.len(), 16);
        assert_eq!(
            notes.path,
            &[
                Move::Forward(10),
                Move::TurnRight,
                Move::Forward(5),
                Move::TurnLeft,
                Move::Forward(5),
                Move::TurnRight,
                Move::Forward(10),
                Move::TurnLeft,
                Move::Forward(4),
                Move::TurnRight,
                Move::Forward(5),
                Move::TurnLeft,
                Move::Forward(5)
            ]
        );
    }

    #[test]
    fn part1() {
        let input = include_str!("../data/example.txt");
        let notes = parse(input).unwrap();
        let task = Task { notes };
        assert_eq!(task.password(), 6032);
    }

    #[test]
    fn input() {
        let input = include_str!("../data/input.txt");
        let notes = parse(input).unwrap();

        let path = notes.path.clone();
        let n = path.len();
        assert_eq!(path[0], Move::Forward(47));
        assert_eq!(path[n - 2], Move::TurnLeft);
        assert_eq!(path[n - 1], Move::Forward(37));

        let task = Task { notes };
        assert_eq!(task.password(), 1428);
    }
}
