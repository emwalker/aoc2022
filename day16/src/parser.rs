use std::fmt::Debug;

use color_eyre::{eyre::eyre, Report, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::multispace1,
    combinator::{all_consuming, map},
    multi::separated_list1,
    sequence::{preceded, tuple},
    Finish, IResult,
};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Name(pub [u8; 2]);

impl Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [a, b] = self.0;
        write!(f, "{}{}", a as char, b as char)
    }
}

impl TryFrom<&str> for Name {
    type Error = Report;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.as_bytes().try_into()?))
    }
}

pub const MAX_NAME: usize = 26_usize.pow(2);

impl Name {
    /// Returns this name as a usize between 0 and 26**2
    pub fn as_usize(self) -> usize {
        let [a, b] = self.0;
        debug_assert!(a.is_ascii_uppercase());
        debug_assert!(b.is_ascii_uppercase());

        (a - b'A') as usize * 26 + (b - b'A') as usize
    }

    /// Returns a name from a usize between 0 and 26**2
    pub fn from_usize(index: usize) -> Self {
        debug_assert!(index < MAX_NAME);
        let a = (index / 26) as u8 + b'A';
        let b = (index % 26) as u8 + b'A';
        Self([a, b])
    }
}

#[derive(Clone, Debug)]
pub struct Valve {
    pub name: Name,
    pub flow: u64,
    pub links: Vec<Name>,
}

#[derive(Debug)]
pub struct Output(pub(crate) Vec<Valve>);

impl Output {
    pub fn iter(&self) -> impl Iterator<Item = &Valve> {
        self.0.iter()
    }
}

fn parse_name(s: &str) -> IResult<&str, Name> {
    map(take(2usize), |slice: &str| {
        Name(slice.as_bytes().try_into().unwrap())
    })(s)
}

fn parse_valve_list(s: &str) -> IResult<&str, Vec<Name>> {
    preceded(
        tag("tunnels lead to valves "),
        separated_list1(tag(", "), parse_name),
    )(s)
}

fn parse_single_valve(s: &str) -> IResult<&str, Vec<Name>> {
    map(preceded(tag("tunnel leads to valve "), parse_name), |v| {
        vec![v]
    })(s)
}

fn parse_reading(s: &str) -> IResult<&str, Valve> {
    map(
        tuple((
            preceded(tag("Valve "), parse_name),
            preceded(tag(" has flow rate="), nom::character::complete::u64),
            preceded(tag("; "), alt((parse_valve_list, parse_single_valve))),
        )),
        |(name, flow, links)| Valve { name, flow, links },
    )(s)
}

pub fn parse(input: &str) -> Result<Output> {
    let readings = all_consuming(separated_list1(multispace1, parse_reading))(input.trim())
        .finish()
        .or(Err(eyre!("failed to parse input")))?
        .1;
    Ok(Output(readings))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Name {
        s.try_into().unwrap()
    }

    #[test]
    fn valve_list() {
        let valve = parse_reading("Valve AA has flow rate=0; tunnels lead to valves DD, II, BB")
            .unwrap()
            .1;

        assert_eq!(valve.name, v("AA"));
        assert_eq!(valve.flow, 0);
        assert_eq!(valve.links, vec![v("DD"), v("II"), v("BB")]);
    }

    #[test]
    fn single_valve() {
        let valve = parse_reading("Valve JJ has flow rate=21; tunnel leads to valve II")
            .unwrap()
            .1;

        assert_eq!(valve.name, v("JJ"));
        assert_eq!(valve.flow, 21);
        assert_eq!(valve.links, vec![v("II")]);
    }
}
