use color_eyre::{self, Report, Result};
use std::str::FromStr;

pub mod naive;

#[derive(Clone, Copy, Debug)]
enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

type Int = i16;

#[derive(Debug, Clone, Copy)]
struct Point([Int; 3]);

impl FromStr for Point {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let a: [Int; 3] = s
            .split(',')
            .map(|v| v.trim().parse::<Int>().expect("an integer"))
            .collect::<Vec<Int>>()
            .try_into()
            .expect("an array");

        Ok(Self(a))
    }
}

#[derive(Debug)]
pub struct Input(Vec<Point>);

impl FromStr for Input {
    type Err = Report;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let vec = value
            .trim()
            .lines()
            .map(|l| l.trim().parse::<Point>())
            .collect::<Result<Vec<Point>>>()?;

        Ok(Self(vec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_good() {
        macro_rules! check {
            ($name:ident) => {
                for input in [include_str!("../data/input.txt")] {
                    assert_eq!(
                        naive::parse(input).unwrap().surface_area(),
                        crate::$name::parse(input).unwrap().surface_area()
                    );
                }
            };
        }

        // Placeholder
        check!(naive);
    }
}
