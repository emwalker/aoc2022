use color_eyre::{eyre::eyre, Report, Result};
use lazy_static::lazy_static;
use nom::{
    branch::alt,
    character::complete::{char, multispace0, multispace1},
    combinator::{all_consuming, map},
    multi::{many1, separated_list0},
    sequence::{delimited, tuple},
    Finish, IResult,
};
use std::{fmt::Debug, str::FromStr};

#[derive(Clone, Eq, PartialEq)]
pub enum Item {
    Number(u16),
    List(Vec<Item>),
}

impl Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => f.write_fmt(format_args!("{n}")),
            Self::List(list) => f.debug_list().entries(list).finish(),
        }
    }
}

impl PartialOrd for Item {
    // https://fasterthanli.me/series/advent-of-code-2022/part-13
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            (l, r) => l.with_slice(|l| r.with_slice(|r| l.partial_cmp(r))),
        }
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Item {
    fn with_slice<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&[Self]) -> T,
    {
        match self {
            Self::List(list) => f(&list[..]),
            Self::Number(n) => f(&[Self::Number(*n)]),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Packet(Item);

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl FromStr for Packet {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let packet = all_consuming(parse_packet)(s)
            .finish()
            .or(Err(eyre!("failed to parse input")))?
            .1;
        Ok(packet)
    }
}

impl Packet {
    pub fn dividers<'d>() -> &'d [Self] {
        lazy_static! {
            static ref DIVIDERS: Vec<Packet> = {
                vec![
                    Packet::from_str("[[2]]").unwrap(),
                    Packet::from_str("[[6]]").unwrap(),
                ]
            };
        }
        &DIVIDERS
    }

    pub fn is_divider(&self) -> bool {
        Self::dividers().contains(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pair {
    pub left: Packet,
    pub right: Packet,
}

impl Pair {
    pub fn is_sorted(&self) -> bool {
        self.left.le(&self.right)
    }
}

#[derive(Clone, Debug)]
pub struct Signal(pub Vec<Pair>);

impl Signal {
    pub fn iter(&self) -> impl Iterator<Item = &Packet> + '_ {
        self.0.iter().flat_map(|pair| vec![&pair.left, &pair.right])
    }
}

fn parse_number(i: &str) -> IResult<&str, Item> {
    map(nom::character::complete::u16, Item::Number)(i)
}

fn parse_list(i: &str) -> IResult<&str, Item> {
    map(
        delimited(
            char('['),
            separated_list0(char(','), alt((parse_number, parse_list))),
            char(']'),
        ),
        Item::List,
    )(i)
}

fn parse_packet(i: &str) -> IResult<&str, Packet> {
    map(parse_list, Packet)(i)
}

fn parse_pair(i: &str) -> IResult<&str, Pair> {
    map(
        tuple((parse_packet, multispace1, parse_packet, multispace0)),
        |(left, _, right, _)| Pair { left, right },
    )(i)
}

fn parse_signal(i: &str) -> IResult<&str, Signal> {
    map(many1(parse_pair), Signal)(i)
}

pub fn parse(input: &str) -> Result<Signal> {
    let input = input.trim();
    let (_rest, signal) = all_consuming(parse_signal)(input)
        .finish()
        .or(Err(eyre!("failed to parse input")))?;
    Ok(signal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal() {
        let input = "\
        [1,1,3,1,1]
        [1,1,5,1,1]

        [[1],[2,3,4]]
        [[1],4]

        [9]
        [[8,7,6]]

        [[4,4],4,4]
        [[4,4],4,4,4]

        [7,7,7,7]
        [7,7,7]

        []
        [3]

        [[[]]]
        [[]]

        [1,[2,[3,[4,[5,6,7]]]],8,9]
        [1,[2,[3,[4,[5,6,0]]]],8,9]";

        let signal = parse(input).unwrap();
        assert_eq!(signal.0.len(), 8);
    }

    #[test]
    fn simple_packet() {
        let (_s, packet) = parse_packet("[1,1,3,1,1]").unwrap();
        let item = Item::List(vec![
            Item::Number(1),
            Item::Number(1),
            Item::Number(3),
            Item::Number(1),
            Item::Number(1),
        ]);
        assert_eq!(Packet(item), packet);
    }

    #[test]
    fn empty_lists() {
        let (_s, list) = parse_list("[[]]").unwrap();
        assert_eq!(Item::List(vec![Item::List(vec![])]), list);
    }

    #[test]
    fn packet_with_a_list() {
        let (_s, packet) = parse_packet("[[1],4]").unwrap();
        let item = Item::List(vec![Item::List(vec![Item::Number(1)]), Item::Number(4)]);
        assert_eq!(Packet(item), packet);
    }

    #[test]
    fn pair() {
        let input = "\
        [1,1]
        [1,5]";
        let (_s, pair) = parse_pair(input).unwrap();

        let left = Packet(Item::List(vec![Item::Number(1), Item::Number(1)]));
        let right = Packet(Item::List(vec![Item::Number(1), Item::Number(5)]));

        assert_eq!(Pair { left, right }, pair);
    }

    fn is_sorted(input: &str) -> bool {
        parse_pair(input).unwrap().1.is_sorted()
    }

    #[test]
    fn ordering() {
        assert!(is_sorted("[1,1,3,1,1]\n[1,1,5,1,1]"));
        assert!(!is_sorted("[1,1,5,1,1]\n[1,1,3,1,1]"));
        assert!(!is_sorted("[7,7,7,7]\n[7,7,7]"));
        assert!(is_sorted("[[4,4],4,4]\n[[4,4],4,4,4]"));

        // Less obvious cases
        assert!(is_sorted(
            "[[[[6,10,5,5],9],6,[[5,2,6,2],9,6,[9,10,6,1,7],4]]]\n[[9,[10],0],[[1]]]"
        ));
        assert!(is_sorted("[[[2,5,[9],[],8],10,2,2,0],[],[]]\n[[9]]"));
        assert!(!is_sorted(
            "[[[10,[6,6],[8],[4,7,0],[8,10,8]],8],[[],3]]
             [[10,[[9,10,0],2]],[],[6,[[],3,[0,5]],3,5],[[[9]],1],[[5,[9,0,4,9],[5,7,8]]]]"
        ));
    }

    #[test]
    fn dividers() {
        assert!(parse_packet("[[2]]").unwrap().1.is_divider());
        assert!(parse_packet("[[6]]").unwrap().1.is_divider());
        assert!(!parse_packet("[2]").unwrap().1.is_divider());
    }
}
