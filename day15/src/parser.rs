use std::ops::RangeInclusive;

use color_eyre::{eyre::eyre, Result};
use nom::{
    bytes::complete::tag,
    combinator::{all_consuming, map},
    sequence::tuple,
    Finish, IResult,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Range(RangeInclusive<i32>);

impl Ord for Range {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.end().cmp(other.0.end())
    }
}

impl PartialOrd for Range {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Range {
    pub fn new(s: i32, e: i32) -> Self {
        Self(s..=e)
    }

    pub fn end(&self) -> &i32 {
        self.0.end()
    }

    pub fn merge(&self, other: &Self) -> Self {
        let (&s1, &e1) = (self.start(), self.end());
        let (&s2, &e2) = (other.start(), other.end());
        Self::new(s1.min(s2), e1.max(e2))
    }

    pub fn overlap(&self, other: &Self) -> bool {
        self.start().max(other.start()) <= self.end().min(other.end())
    }

    pub fn start(&self) -> &i32 {
        self.0.start()
    }
}

pub struct Reading {
    pub sensor: Point,
    pub beacon: Point,
    pub distance: i32,
}

impl From<(i32, i32, i32, i32)> for Reading {
    fn from((sx, sy, bx, by): (i32, i32, i32, i32)) -> Self {
        let sensor = Point::new(sx, sy);
        let beacon = Point::new(bx, by);
        let distance = (bx - sx).abs() + (by - sy).abs();

        Self {
            sensor,
            beacon,
            distance,
        }
    }
}

impl Reading {
    pub fn range_at_y(&self, y: i32) -> (Option<Range>, Option<Point>) {
        let d = self.distance - (y - self.sensor.y).abs();
        if d <= 0 {
            return (None, None);
        }

        let beacon = if self.beacon.y == y {
            Some(self.beacon)
        } else {
            None
        };

        let r = Range::new(self.sensor.x - d, self.sensor.x + d);
        (Some(r), beacon)
    }
}

pub fn parse_reading(s: &str) -> IResult<&str, Reading> {
    map(
        tuple((
            tag("Sensor at x="),
            nom::character::complete::i32,
            tag(", y="),
            nom::character::complete::i32,
            tag(": closest beacon is at x="),
            nom::character::complete::i32,
            tag(", y="),
            nom::character::complete::i32,
        )),
        |(_, sx, _, sy, _, bx, _, by)| Reading::from((sx, sy, bx, by)),
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
        assert_eq!(
            r.range_at_y(10),
            (Some(Range::new(2, 14)), Some(Point::new(2, 10)))
        );

        let r = reading("Sensor at x=2, y=18: closest beacon is at x=-2, y=15").unwrap();
        assert_eq!(
            r.range_at_y(15),
            (Some(Range::new(-2, 6)), Some(Point::new(-2, 15)))
        );
        assert_eq!(r.range_at_y(16), (Some(Range::new(-3, 7)), None));
        assert_eq!(r.range_at_y(100), (None, None));
    }

    #[test]
    fn merge() {
        assert_eq!(Range::new(0, 1).merge(&Range::new(1, 2)), Range::new(0, 2));
        assert_eq!(Range::new(0, 5).merge(&Range::new(1, 6)), Range::new(0, 6));
    }

    #[test]
    fn overlap() {
        assert!(Range::new(0, 1).overlap(&Range::new(1, 2)));
        assert!(!Range::new(0, 1).overlap(&Range::new(2, 3)));
    }
}
