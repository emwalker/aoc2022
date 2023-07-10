use color_eyre::{eyre::eyre, Report};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::multispace1,
    combinator::{all_consuming, map},
    multi::separated_list1,
    sequence::{preceded, tuple},
    Finish, IResult,
};
use std::{collections::HashMap, str::FromStr};

pub mod naive;

pub const EXAMPLE: &str = "\
root: pppw + sjmn
dbpl: 5
cczh: sllz + lgvd
zczc: 2
ptdq: humn - dvpt
dvpt: 3
lfqf: 4
humn: 5
ljgn: 2
sjmn: drzm * dbpl
sllz: 4
pppw: cczh / lfqf
lgvd: ljgn * ptdq
drzm: hmdt - zczc
hmdt: 32";

pub type Int = i64;

#[derive(Debug)]
pub enum Step {
    Shout(Int),
    Add(String, String),
    Mul(String, String),
    Sub(String, String),
    Div(String, String),
}

#[derive(Debug)]
pub struct Input(HashMap<String, Step>);

fn parse_shout(i: &str) -> IResult<&str, (String, Step)> {
    map(
        tuple((
            take_till1(|c| c == ':'),
            tag(":"),
            multispace1,
            nom::character::complete::i64,
        )),
        |(name, _, _, n): (&str, &str, &str, Int)| (name.to_owned(), Step::Shout(n as _)),
    )(i)
}

fn parse_add(i: &str) -> IResult<&str, (String, Step)> {
    map(
        tuple((
            take_till1(|c| c == ':'),
            tag(":"),
            multispace1,
            take_till1(|c| c == ' '),
            preceded(tag(" + "), take_till1(|c| c == '\n')),
        )),
        |(name, _, _, lhs, rhs): (&str, &str, &str, &str, &str)| {
            (name.to_owned(), Step::Add(lhs.to_owned(), rhs.to_owned()))
        },
    )(i)
}

fn parse_mul(i: &str) -> IResult<&str, (String, Step)> {
    map(
        tuple((
            take_till1(|c| c == ':'),
            tag(":"),
            multispace1,
            take_till1(|c| c == ' '),
            preceded(tag(" * "), take_till1(|c| c == '\n')),
        )),
        |(name, _, _, lhs, rhs): (&str, &str, &str, &str, &str)| {
            (name.to_owned(), Step::Mul(lhs.to_owned(), rhs.to_owned()))
        },
    )(i)
}

fn parse_sub(i: &str) -> IResult<&str, (String, Step)> {
    map(
        tuple((
            take_till1(|c| c == ':'),
            tag(":"),
            multispace1,
            take_till1(|c| c == ' '),
            preceded(tag(" - "), take_till1(|c| c == '\n')),
        )),
        |(name, _, _, lhs, rhs): (&str, &str, &str, &str, &str)| {
            (name.to_owned(), Step::Sub(lhs.to_owned(), rhs.to_owned()))
        },
    )(i)
}

fn parse_div(i: &str) -> IResult<&str, (String, Step)> {
    map(
        tuple((
            take_till1(|c| c == ':'),
            tag(":"),
            multispace1,
            take_till1(|c| c == ' '),
            preceded(tag(" / "), take_till1(|c| c == '\n')),
        )),
        |(name, _, _, lhs, rhs): (&str, &str, &str, &str, &str)| {
            (name.to_owned(), Step::Div(lhs.to_owned(), rhs.to_owned()))
        },
    )(i)
}

fn parse_step(i: &str) -> IResult<&str, (String, Step)> {
    alt((parse_shout, parse_add, parse_mul, parse_sub, parse_div))(i)
}

impl FromStr for Input {
    type Err = Report;

    fn from_str(i: &str) -> Result<Self, Self::Err> {
        let (_s, steps) = all_consuming(separated_list1(multispace1, parse_step))(i.trim())
            .finish()
            .or(Err(eyre!("failed to parse input")))?;
        let map = steps.into_iter().collect::<HashMap<String, _>>();
        Ok(Self(map))
    }
}

impl Input {
    pub fn get(&self, key: &str) -> Option<&Step> {
        self.0.get(key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let input = EXAMPLE.parse::<Input>().unwrap();

        assert_eq!(input.len(), 15);
        assert!(matches!(input.get("root"), Some(Step::Add(_, _))));
        assert!(matches!(input.get("drzm"), Some(Step::Sub(_, _))));
        assert!(matches!(input.get("hmdt"), Some(Step::Shout(_))));
    }
}
