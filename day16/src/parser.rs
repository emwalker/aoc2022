use std::fmt::Debug;

use color_eyre::{eyre::eyre, Report, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace1},
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

#[derive(Clone, Debug)]
pub struct Valve {
    pub name: String,
    pub flow: u8,
    pub links: Vec<String>,
}

pub type Valves = Vec<Valve>;

fn parse_valve_list(s: &str) -> IResult<&str, Vec<String>> {
    preceded(
        tag("tunnels lead to valves "),
        separated_list1(tag(", "), map(alphanumeric1, str::to_owned)),
    )(s)
}

fn parse_single_valve(s: &str) -> IResult<&str, Vec<String>> {
    map(
        preceded(
            tag("tunnel leads to valve "),
            map(alphanumeric1, str::to_owned),
        ),
        |v| vec![v],
    )(s)
}

fn parse_reading(s: &str) -> IResult<&str, Valve> {
    map(
        tuple((
            preceded(tag("Valve "), alphanumeric1),
            preceded(tag(" has flow rate="), nom::character::complete::u8),
            preceded(tag("; "), alt((parse_valve_list, parse_single_valve))),
        )),
        |(name, flow, links)| Valve {
            name: name.to_owned(),
            flow,
            links,
        },
    )(s)
}

pub fn parse(input: &str) -> Result<Valves> {
    let readings = all_consuming(separated_list1(multispace1, parse_reading))(input.trim())
        .finish()
        .or(Err(eyre!("failed to parse input")))?
        .1;
    Ok(readings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valve_list() {
        let valve = parse_reading("Valve AA has flow rate=0; tunnels lead to valves DD, II, BB")
            .unwrap()
            .1;

        assert_eq!(valve.name, "AA");
        assert_eq!(valve.flow, 0);
        assert_eq!(valve.links, vec!["DD", "II", "BB"]);
    }

    #[test]
    fn single_valve() {
        let valve = parse_reading("Valve JJ has flow rate=21; tunnel leads to valve II")
            .unwrap()
            .1;

        assert_eq!(valve.name, "JJ");
        assert_eq!(valve.flow, 21);
        assert_eq!(valve.links, vec!["II"]);
    }
}
