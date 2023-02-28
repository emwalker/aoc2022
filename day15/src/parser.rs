use std::ops::{BitAnd, BitOr, RangeInclusive};

use color_eyre::{eyre::eyre, Result};
use nom::{
    bytes::complete::tag,
    combinator::{all_consuming, map},
    sequence::{preceded, separated_pair},
    Finish, IResult,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Point {
    x: i64,
    y: i64,
}

impl Point {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn manhattan_distance(&self, other: &Self) -> i64 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    pub fn tuning_frequency(&self) -> i64 {
        self.x * 4_000_000 + self.y
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Range(pub RangeInclusive<i64>);

impl Ord for Range {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.start().cmp(other.0.start()).reverse()
    }
}

impl PartialOrd for Range {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl BitAnd for Range {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let (&s1, &e1) = (self.start(), self.end());
        let (&s2, &e2) = (rhs.start(), rhs.end());
        Self::new(s1.max(s2), e1.min(e2))
    }
}

impl BitOr for Range {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let (&s1, &e1) = (self.start(), self.end());
        let (&s2, &e2) = (rhs.start(), rhs.end());
        Self::new(s1.min(s2), e1.max(e2))
    }
}

impl Range {
    pub fn new(s: i64, e: i64) -> Self {
        Self(s..=e)
    }

    pub fn end(&self) -> &i64 {
        self.0.end()
    }

    pub fn overlap(&self, other: &Self) -> bool {
        self.start().max(other.start()) <= self.end().min(other.end())
    }

    pub fn start(&self) -> &i64 {
        self.0.start()
    }
}

pub struct Reading {
    pub sensor: Point,
    pub beacon: Point,
    pub distance: i64,
}

impl From<(i64, i64, i64, i64)> for Reading {
    fn from((sx, sy, bx, by): (i64, i64, i64, i64)) -> Self {
        let sensor = Point::new(sx, sy);
        let beacon = Point::new(bx, by);
        let distance = sensor.manhattan_distance(&beacon);

        Self {
            sensor,
            beacon,
            distance,
        }
    }
}

impl Reading {
    pub fn range_at_y(&self, y: i64) -> Option<Range> {
        let d = self.distance - (y - self.sensor.y).abs();
        if d <= 0 {
            return None;
        }

        let r = Range::new(self.sensor.x - d, self.sensor.x + d);
        Some(r)
    }
}

fn parse_point(s: &str) -> IResult<&str, Point> {
    map(
        separated_pair(
            preceded(tag("x="), nom::character::complete::i64),
            tag(", "),
            preceded(tag("y="), nom::character::complete::i64),
        ),
        |(x, y)| Point { x, y },
    )(s)
}

pub fn parse_reading(s: &str) -> IResult<&str, Reading> {
    map(
        separated_pair(
            preceded(tag("Sensor at "), parse_point),
            tag(": closest beacon is at "),
            parse_point,
        ),
        |(sensor, beacon)| Reading {
            sensor,
            beacon,
            distance: sensor.manhattan_distance(&beacon),
        },
    )(s)
}

pub fn reading(input: &str) -> Result<Reading> {
    let reading = all_consuming(parse_reading)(input)
        .finish()
        .or(Err(eyre!("problem parsing reading")))?
        .1;
    Ok(reading)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_case() {
        let r = reading("Sensor at x=2, y=18: closest beacon is at x=-2, y=15").unwrap();
        assert_eq!(r.sensor, Point::new(2, 18));
        assert_eq!(r.beacon, Point::new(-2, 15));
    }

    #[test]
    fn range() {
        let r = reading("Sensor at x=8, y=7: closest beacon is at x=2, y=10").unwrap();
        assert_eq!(r.range_at_y(10), Some(Range::new(2, 14)));

        let r = reading("Sensor at x=2, y=18: closest beacon is at x=-2, y=15").unwrap();
        assert_eq!(r.range_at_y(15), Some(Range::new(-2, 6)),);
        assert_eq!(r.range_at_y(16), Some(Range::new(-3, 7)));
        assert_eq!(r.range_at_y(100), None);
    }

    #[test]
    fn merge() {
        assert_eq!(Range::new(0, 1) | Range::new(1, 2), Range::new(0, 2));
        assert_eq!(Range::new(0, 5) | Range::new(1, 6), Range::new(0, 6));
    }

    #[test]
    fn overlap() {
        assert!(Range::new(0, 1).overlap(&Range::new(1, 2)));
        assert!(!Range::new(0, 1).overlap(&Range::new(2, 3)));
    }
}
