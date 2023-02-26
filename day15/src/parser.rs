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
    pub fn range_at_y(&self, y: i32) -> (Option<RangeInclusive<i32>>, Option<Point>) {
        let d = self.distance - (y - self.sensor.y).abs();
        if d <= 0 {
            return (None, None);
        }

        let beacon = if self.beacon.y == y {
            Some(self.beacon)
        } else {
            None
        };

        let r = (self.sensor.x - d)..=(self.sensor.x + d);
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
        assert_eq!(r.range_at_y(10), (Some(2..=14), Some(Point::new(2, 10))));

        let r = reading("Sensor at x=2, y=18: closest beacon is at x=-2, y=15").unwrap();
        assert_eq!(r.range_at_y(15), (Some(-2..=6), Some(Point::new(-2, 15))));
        assert_eq!(r.range_at_y(16), (Some(-3..=7), None));
        assert_eq!(r.range_at_y(100), (None, None));
    }
}
