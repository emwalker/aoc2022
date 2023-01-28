// Heavily copied from Amos's discussion, here:
// https://fasterthanli.me/series/advent-of-code-2022/part-5#reader-suggestion-use-nom-s-number-parser

use color_eyre::{self, Result};
use itertools::Itertools;
use std::{
    fmt::{Debug, Display, Write},
    io,
};

#[derive(Copy, Clone)]
pub struct Crate(char);

impl Debug for Crate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.0)
    }
}

impl Display for Crate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.0)
    }
}

#[derive(Debug)]
pub struct Instruction {
    count: usize,
    src: usize,
    dst: usize,
}

struct CrateMover9000;

impl CrateMover9000 {
    fn apply(ins: &Instruction, stacks: &mut Stacks) {
        for _ in 0..ins.count {
            let el = stacks.0[ins.src].pop().unwrap();
            stacks.0[ins.dst].push(el);
        }
    }
}

struct CrateMover9001;

impl CrateMover9001 {
    fn apply(ins: &Instruction, stacks: &mut Stacks) {
        for krate in (0..ins.count)
            .map(|_| stacks.0[ins.src].pop().unwrap())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            stacks.0[ins.dst].push(krate);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stacks(pub Vec<Vec<Crate>>);

impl Stacks {
    fn top_crates(&self) -> String {
        self.0.iter().map(|stack| stack.last().unwrap()).join("")
    }

    #[allow(unused)]
    fn len(&self) -> usize {
        self.0.len()
    }
}

mod parser {
    use std::collections::{HashMap, VecDeque};

    use super::*;
    use nom::{
        branch::alt,
        bytes::complete::{tag, take},
        combinator::{all_consuming, map},
        multi::separated_list1,
        sequence::{delimited, preceded, tuple},
        Finish, IResult,
    };

    fn parse_crate(i: &str) -> IResult<&str, Crate> {
        let first_char = |s: &str| Crate(s.chars().next().unwrap());
        let f = delimited(tag("["), take(1_usize), tag("]"));
        map(f, first_char)(i)
    }

    fn parse_hole(i: &str) -> IResult<&str, ()> {
        map(tag("   "), drop)(i)
    }

    fn parse_crate_or_hole(i: &str) -> IResult<&str, Option<Crate>> {
        alt((map(parse_crate, Some), map(parse_hole, |_| None)))(i)
    }

    fn parse_crate_line(i: &str) -> IResult<&str, Vec<Option<Crate>>> {
        separated_list1(tag(" "), parse_crate_or_hole)(i)
    }

    #[derive(Debug)]
    pub struct Ast(Vec<Vec<Option<Crate>>>);

    impl Ast {
        pub fn finalize(self) -> Stacks {
            let Self(crates) = self;
            assert!(!crates.is_empty());

            // Convert rows of crates to stacks
            let mut cols: HashMap<usize, VecDeque<Crate>> = HashMap::new();

            for row in crates {
                for (i, c) in row.into_iter().enumerate() {
                    if let Some(c) = c {
                        cols.entry(i).or_insert(VecDeque::new()).push_front(c);
                    }
                }
            }

            let indexes: Vec<_> = cols.keys().sorted().cloned().collect();
            let stacks: Vec<_> = indexes
                .iter()
                .map(|i| cols.remove(i).unwrap().into_iter().collect_vec())
                .collect();

            Stacks(stacks)
        }
    }

    pub fn parse_stacks<I>(it: &mut I) -> Result<Ast>
    where
        I: Iterator<Item = String>,
    {
        let crates = it
            .map_while(|line| {
                all_consuming(parse_crate_line)(&line)
                    .finish()
                    .ok()
                    .map(|(_, c)| c)
            })
            .collect();

        Ok(Ast(crates))
    }

    fn parse_number(i: &str) -> IResult<&str, usize> {
        map(nom::character::complete::u32, |n| n as _)(i)
    }

    fn parse_pile_number(i: &str) -> IResult<&str, usize> {
        map(parse_number, |i| i - 1)(i)
    }

    fn parse_instruction(i: &str) -> IResult<&str, Instruction> {
        map(
            tuple((
                preceded(tag("move "), parse_number),
                preceded(tag(" from "), parse_pile_number),
                preceded(tag(" to "), parse_pile_number),
            )),
            |(count, src, dst)| Instruction { count, src, dst },
        )(i)
    }

    pub struct Instructions<Iter> {
        iter: Iter,
    }

    impl<Iter> Instructions<Iter> {
        pub fn new(iter: Iter) -> Self {
            Instructions { iter }
        }
    }

    impl<Iter> Iterator for Instructions<Iter>
    where
        Iter: Iterator<Item = String>,
    {
        type Item = Instruction;

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next().and_then(|line| {
                all_consuming(parse_instruction)(&line)
                    .finish()
                    .ok()
                    .map(|(_rest, ins)| ins)
            })
        }
    }

    pub trait InstructionsIterExt: Sized {
        fn instructions(self) -> Instructions<Self>;
    }

    impl<Iter> InstructionsIterExt for Iter {
        fn instructions(self) -> Instructions<Iter> {
            Instructions::new(self)
        }
    }
}

fn main() -> Result<()> {
    use crate::parser::InstructionsIterExt;

    let mut it = io::stdin().lines().flat_map(|l| l.ok());
    let mut cm9000 = parser::parse_stacks(&mut it)?.finalize();
    let mut cm9001 = cm9000.clone();

    // We're expecting a blank line
    assert_eq!(it.next(), Some("".into()));

    for ins in it.instructions() {
        CrateMover9000::apply(&ins, &mut cm9000);
        CrateMover9001::apply(&ins, &mut cm9001);
    }

    println!("CrateMover 9000: {}", cm9000.top_crates());
    println!("CrateMover 9001: {}", cm9001.top_crates());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::InstructionsIterExt;

    const INPUT: &str = "    [D]
[N] [C]
[Z] [M] [P]
 1   2   3

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";

    #[test]
    fn crate_mover_9000() {
        let mut it = INPUT.lines().map(str::to_string);
        let mut stacks = parser::parse_stacks(&mut it).unwrap().finalize();
        assert_eq!(it.next().unwrap(), "");

        for ins in it.instructions() {
            CrateMover9000::apply(&ins, &mut stacks);
        }

        assert_eq!(stacks.top_crates(), "CMZ");
    }

    #[test]
    fn crate_mover_9001() {
        let mut it = INPUT.lines().map(str::to_string);
        let mut stacks = parser::parse_stacks(&mut it).unwrap().finalize();
        assert_eq!(it.next().unwrap(), "");

        for ins in it.instructions() {
            CrateMover9001::apply(&ins, &mut stacks);
        }

        assert_eq!(stacks.top_crates(), "MCD");
    }

    #[test]
    fn parse_input() {
        let mut it = INPUT.lines().map(str::to_string);
        let stacks = parser::parse_stacks(&mut it).unwrap().finalize();
        assert_eq!(stacks.len(), 3);

        // We've consumed the line of crate labels, and now we're at the blank line
        assert_eq!(it.next().unwrap(), "");

        let ins: Vec<_> = it.instructions().collect();
        assert_eq!(ins.len(), 4);
    }
}
