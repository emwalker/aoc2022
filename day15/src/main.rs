#![feature(binary_heap_into_iter_sorted)]

use color_eyre::{self, eyre::eyre, Report, Result};
use std::{
    collections::{BinaryHeap, HashSet},
    io::{self, Read},
    str::FromStr,
    vec,
};

mod parser;
use parser::{Point, Range, Reading};

struct Readings(Vec<Reading>);

impl FromStr for Readings {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(
            s.lines()
                .map(str::trim)
                .map(parser::reading)
                .collect::<Result<Vec<Reading>>>()?,
        ))
    }
}

impl Readings {
    fn iter(&self) -> impl Iterator<Item = &Reading> + '_ {
        self.0.iter()
    }
}

struct Task {
    readings: Readings,
}

impl FromStr for Task {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let readings = s.parse::<Readings>()?;
        Ok(Self { readings })
    }
}

const LENGTH: i32 = 4000000;

impl Task {
    fn coverage_at(&self, y: i32) -> (Vec<Range>, HashSet<Point>) {
        let mut ranges: BinaryHeap<Range> = BinaryHeap::new();
        let mut beacons = HashSet::new();

        for reading in self.readings.iter() {
            let (curr, beacon) = reading.range_at_y(y);

            if let Some(curr) = curr {
                ranges.push(curr);
            }

            if let Some(beacon) = beacon {
                beacons.insert(beacon);
            }
        }

        let ranges: Vec<_> = ranges.into_iter_sorted().collect();
        let mut merged = vec![];

        for curr in ranges.into_iter().rev() {
            if let Some(prev) = merged.pop() {
                if curr.overlap(&prev) {
                    merged.push(prev | curr);
                } else {
                    merged.push(prev);
                    merged.push(curr);
                }
            } else {
                merged.push(curr);
            }
        }

        (merged, beacons)
    }

    fn no_beacon(&self, y: i32) -> i32 {
        let (ranges, beacons) = self.coverage_at(y);
        self.count_elements(ranges.iter()) - beacons.len() as i32
    }

    fn count_elements<'r, R>(&self, ranges: R) -> i32
    where
        R: Iterator<Item = &'r Range>,
    {
        ranges.map(|r| r.end() - r.start() + 1).sum::<i32>()
    }

    fn tuning_frequency(&self, length: i32) -> Result<i128> {
        let l = Range::new(0, length);

        for y in 0..=length {
            let (mut ranges, _) = self.coverage_at(y);

            if ranges.iter().any(|r| r.clone() & l.clone() == l) {
                continue;
            }

            let r = ranges.pop().ok_or(eyre!("expected a range"))?;
            let x = r.start() - 1;
            return Ok(x as i128 * LENGTH as i128 + y as i128);
        }

        Err(eyre!("no tuning frequency found"))
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = input.parse::<Task>()?;
    println!("positions with no beacon: {}", task.no_beacon(2000000));
    println!("tuning frequency: {}", task.tuning_frequency(LENGTH)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn readings() -> Readings {
        let input = include_str!("../data/example.txt");
        input.parse::<Readings>().unwrap()
    }

    #[test]
    fn parsing() {
        let readings = readings();
        assert_eq!(readings.0.len(), 14);
    }

    #[test]
    fn no_beacon() {
        let input = include_str!("../data/example.txt");
        let task = input.parse::<Task>().unwrap();
        assert_eq!(task.no_beacon(10), 26);
    }

    #[test]
    fn no_beacon_with_input() {
        let input = include_str!("../data/input.txt");
        let task = input.parse::<Task>().unwrap();
        assert_eq!(task.no_beacon(2_000_000), 5461729);
    }

    #[test]
    fn coverage_at() {
        let input = include_str!("../data/example.txt");
        let task = input.parse::<Task>().unwrap();
        let merged = task.coverage_at(11).0;
        assert_eq!(merged, vec![Range::new(-3, 13), Range::new(15, 25)]);
    }

    #[test]
    fn tuning_frequency() {
        let input = include_str!("../data/example.txt");
        let task = input.parse::<Task>().unwrap();
        assert_eq!(task.tuning_frequency(20).unwrap(), 56000011);
    }
}
