#![feature(binary_heap_into_iter_sorted)]

use color_eyre::{self, Report, Result};
use std::{
    collections::{BinaryHeap, HashSet, VecDeque},
    io::{self, Read},
    str::FromStr,
};

mod parser;
use parser::{Range, Reading};

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

impl Task {
    fn no_beacon(&self, y: i32) -> i32 {
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

        let mut merged = VecDeque::new();

        while let Some(curr) = ranges.pop() {
            if let Some(prev) = merged.pop_front() {
                if curr.overlap(&prev) {
                    merged.push_front(curr.merge(&prev));
                } else if curr > prev {
                    merged.push_front(prev);
                    merged.push_front(curr);
                } else {
                    merged.push_front(curr);
                    merged.push_front(prev);
                }
            } else {
                merged.push_front(curr);
            }
        }

        merged.iter().map(|r| r.end() - r.start() + 1).sum::<i32>() - beacons.len() as i32
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = input.parse::<Task>()?;
    let positions = task.no_beacon(2000000);
    println!("positions with no beacon: {positions}",);

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
}
