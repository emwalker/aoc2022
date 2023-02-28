#![feature(binary_heap_into_iter_sorted)]

use color_eyre::{self, eyre::eyre, Report, Result};
use itertools::Itertools;
use std::{
    collections::BinaryHeap,
    io::{self, Read},
    str::FromStr,
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

const LENGTH: i64 = 4000000;

impl Task {
    fn clamped_ranges(&self, y: i64, x_range: Range) -> impl Iterator<Item = Range> {
        self.ranges(y).filter_map(move |r| {
            let r = r & x_range.clone();
            if r.start() > r.end() {
                None
            } else {
                Some(r)
            }
        })
    }

    fn ranges(&self, y: i64) -> impl Iterator<Item = Range> {
        let mut ranges: BinaryHeap<Range> = BinaryHeap::new();

        for reading in self.readings.iter() {
            if let Some(curr) = reading.range_at_y(y) {
                ranges.push(curr);
            }
        }

        ranges.into_iter_sorted().coalesce(|a, b| {
            if a.overlap(&b) {
                Ok(a | b)
            } else {
                Err((a, b))
            }
        })
    }

    fn no_beacon(&self, y: i64) -> i64 {
        let ranges = self.ranges(y);
        self.count_elements(ranges)
    }

    fn count_elements<R>(&self, ranges: R) -> i64
    where
        R: Iterator<Item = Range>,
    {
        ranges.map(|r| r.end() - r.start()).sum::<i64>()
    }

    fn hidden_beacon(&self, length: i64) -> Result<Point> {
        let mut y_range = Range::new(0, length);
        let x_range = Range::new(0, length);

        y_range
            .0
            .find_map(|y| {
                self.clamped_ranges(y, x_range.clone())
                    .nth(1)
                    .map(|r| Point::new(r.start() - 1, y))
            })
            .ok_or(eyre!("no beacon found"))
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = input.parse::<Task>()?;
    println!("positions with no beacon: {}", task.no_beacon(2000000));
    println!(
        "tuning frequency: {}",
        task.hidden_beacon(LENGTH)?.tuning_frequency()
    );

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
        let merged = task.ranges(11).collect_vec();
        assert_eq!(merged, vec![Range::new(-3, 13), Range::new(15, 25)]);
    }

    #[test]
    fn tuning_frequency() {
        let input = include_str!("../data/example.txt");
        let task = input.parse::<Task>().unwrap();
        assert_eq!(task.hidden_beacon(20).unwrap().tuning_frequency(), 56000011);
    }

    // #[test]
    #[allow(unused)]
    fn tuning_frequency_with_input() {
        let input = include_str!("../data/input.txt");
        let task = input.parse::<Task>().unwrap();
        assert_eq!(
            task.hidden_beacon(LENGTH).unwrap().tuning_frequency(),
            10621647166538
        );
    }
}
