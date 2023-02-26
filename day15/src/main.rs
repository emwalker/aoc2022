use color_eyre::{self, Report, Result};
use std::{
    collections::HashSet,
    io::{self, Read},
    ops::RangeInclusive,
    str::FromStr,
};

mod parser;
use parser::Reading;

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
        let mut ranges: Vec<RangeInclusive<i32>> = vec![];
        let mut beacons = HashSet::new();

        for reading in self.readings.iter() {
            let (curr, beacon) = reading.range_at_y(y);

            if let Some(curr) = curr {
                if let Some(prev) = ranges.pop() {
                    let (&s1, &e1) = (prev.start(), prev.end());
                    let (&s2, &e2) = (curr.start(), curr.end());
                    ranges.push(s1.min(s2)..=e1.max(e2));
                } else {
                    ranges.push(curr);
                }
            }

            if let Some(beacon) = beacon {
                beacons.insert(beacon);
            }
        }

        ranges.iter().map(|r| r.end() - r.start() + 1).sum::<i32>() - beacons.len() as i32
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
