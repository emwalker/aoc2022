use color_eyre::{eyre::eyre, Result};
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::{all_consuming, map},
    multi::{fold_many1, separated_list1},
    sequence::{separated_pair, tuple},
    Finish, IResult,
};
use std::{fmt::Debug, ops::RangeInclusive};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({},{})", self.x, self.y))
    }
}

#[derive(Eq, PartialEq)]
pub struct Wall {
    pub p0: Point,
    pub p1: Point,
    pub xrange: RangeInclusive<i32>,
    pub yrange: RangeInclusive<i32>,
}

impl Wall {
    pub fn new(p0: Point, p1: Point) -> Option<Self> {
        let (xlb, xub) = [p0.x, p1.x].into_iter().sorted().collect_tuple()?;
        let xrange = xlb..=xub;

        let (ylb, yub) = [p0.y, p1.y].into_iter().sorted().collect_tuple()?;
        let yrange = ylb..=yub;

        Some(Self {
            p0,
            p1,
            xrange,
            yrange,
        })
    }
}

impl Debug for Wall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{:?},{:?}]", self.p0, self.p1))
    }
}

#[derive(Debug)]
pub struct Cave(Vec<Wall>);

impl Cave {
    pub fn iter(&self) -> impl Iterator<Item = &Wall> + '_ {
        self.0.iter()
    }
}

fn parse_coord(i: &str) -> IResult<&str, Point> {
    map(
        separated_pair(
            nom::character::complete::i32,
            nom::character::complete::char(','),
            nom::character::complete::i32,
        ),
        |(x, y)| Point { x, y },
    )(i)
}

fn parse_walls(i: &str) -> IResult<&str, Vec<Wall>> {
    let f = map(separated_list1(tag(" -> "), parse_coord), |coords| {
        coords
            .into_iter()
            .tuple_windows()
            .map(|(from, to)| Wall::new(from, to).unwrap())
            .collect_vec()
    });

    map(tuple((f, multispace0)), |(walls, _)| walls)(i)
}

fn parse_cave(i: &str) -> IResult<&str, Cave> {
    map(
        fold_many1(parse_walls, Vec::new, |mut acc, walls| {
            acc.extend(walls);
            acc
        }),
        Cave,
    )(i)
}

pub fn parse(input: &str) -> Result<Cave> {
    let cave = all_consuming(parse_cave)(input.trim())
        .finish()
        .or(Err(eyre!("failed to parse input: {input}")))?
        .1;
    Ok(cave)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(input: &str) -> Cave {
        parse(input).unwrap()
    }

    #[test]
    fn parsing() {
        let input = "\
        498,4 -> 498,6 -> 496,6
        503,4 -> 502,4 -> 502,9 -> 494,9";

        let cave = state(input);
        assert_eq!(cave.0.len(), 5);
    }

    #[test]
    fn walls() {
        let (_s, walls) = parse_walls("498,4 -> 498,6 -> 496,6").unwrap();

        assert_eq!(walls.len(), 2);

        assert_eq!(
            walls[0],
            Wall::new(Point { x: 498, y: 4 }, Point { x: 498, y: 6 }).unwrap(),
        );

        assert_eq!(
            walls[1],
            Wall::new(Point { x: 498, y: 6 }, Point { x: 496, y: 6 }).unwrap(),
        );
    }
}
