use color_eyre::{self, Report, Result};
use std::str::FromStr;

pub mod dfs1;
pub mod dfs2;
pub mod naive;

#[derive(Clone, Copy, Debug)]
enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

type Int = i16;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Cube {
    x: Int,
    y: Int,
    z: Int,
}

impl FromStr for Cube {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let a = s
            .split(',')
            .map(|v| v.trim().parse::<Int>().expect("an integer"))
            .collect::<Vec<Int>>();

        Ok(Self {
            x: a[0],
            y: a[1],
            z: a[2],
        })
    }
}

impl Cube {
    fn adjacent(&self, other: &Self) -> bool {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y) + self.z.abs_diff(other.z) == 1
    }

    fn shift(&self, (dx, dy, dz): (Int, Int, Int)) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z + dz,
        }
    }
}

#[derive(Debug)]
pub struct Input(Vec<Cube>);

impl FromStr for Input {
    type Err = Report;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let vec = value
            .trim()
            .lines()
            .map(|l| l.trim().parse::<Cube>())
            .collect::<Result<Vec<Cube>>>()?;

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

        check!(dfs1);
    }
}
