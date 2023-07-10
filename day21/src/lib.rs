use color_eyre::{eyre::eyre, Report};
use core::panic;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, multispace1},
    combinator::{all_consuming, map},
    multi::separated_list1,
    sequence::{separated_pair, tuple},
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

#[derive(Debug, Clone)]
pub enum Step {
    Shout(Int),
    Add(String, String),
    Mul(String, String),
    Sub(String, String),
    Div(String, String),
}

#[derive(Debug)]
pub struct Input(HashMap<String, Step>);

fn parse_expression(i: &str) -> IResult<&str, Step> {
    alt((
        map(nom::character::complete::i64, Step::Shout),
        map(
            tuple((
                alpha1,
                alt((tag(" + "), tag(" - "), tag(" * "), tag(" / "))),
                alpha1,
            )),
            |(lhs, op, rhs)| match op {
                " + " => Step::Add(lhs.into(), rhs.into()),
                " - " => Step::Sub(lhs.into(), rhs.into()),
                " * " => Step::Mul(lhs.into(), rhs.into()),
                " / " => Step::Div(lhs.into(), rhs.into()),
                _ => panic!("bad operator: {op}"),
            },
        ),
    ))(i)
}

fn parse_step(i: &str) -> IResult<&str, (&str, Step)> {
    separated_pair(alpha1, tag(": "), parse_expression)(i)
}

impl FromStr for Input {
    type Err = Report;

    fn from_str(i: &str) -> Result<Self, Self::Err> {
        let (s, steps) = all_consuming(separated_list1(multispace1, parse_step))(i.trim())
            .finish()
            .or(Err(eyre!("failed to parse input")))?;
        assert!(s.is_empty());

        let map = steps
            .into_iter()
            .map(|(n, s)| (n.to_owned(), s))
            .collect::<HashMap<String, _>>();

        Ok(Self(map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let input = EXAMPLE.parse::<Input>().unwrap();

        assert_eq!(input.0.len(), 15);
        assert!(matches!(input.0.get("root"), Some(Step::Add(_, _))));
        assert!(matches!(input.0.get("drzm"), Some(Step::Sub(_, _))));
        assert!(matches!(input.0.get("hmdt"), Some(Step::Shout(_))));
    }
}
