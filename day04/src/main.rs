use std::io;
use std::str::FromStr;

use color_eyre::{self, Report, Result};

use color_eyre::eyre::eyre;
use itertools::Itertools;
use std::ops;

trait ElfRange {
    fn superset(&self, other: &Self) -> bool;

    fn contains_or_is_contained(&self, other: &Self) -> bool;

    fn overlaps(&self, other: &Self) -> bool;
}

impl ElfRange for ops::RangeInclusive<u32> {
    fn superset(&self, other: &Self) -> bool {
        self.contains(other.start()) && self.contains(other.end())
    }

    fn contains_or_is_contained(&self, other: &Self) -> bool {
        self.superset(other) || other.superset(self)
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.contains(other.start()) || other.contains(self.start())
    }
}

struct RangeIn(ops::RangeInclusive<u32>);

impl FromStr for RangeIn {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Some((s, e)) = s.split('-').tuples().next() {
            let s: u32 = s.parse()?;
            let e: u32 = e.parse()?;
            return Ok(Self(s..=e));
        }

        Err(eyre!("bad input"))
    }
}

impl RangeIn {
    fn contains_or_is_contained(&self, other: &Self) -> bool {
        self.0.contains_or_is_contained(&other.0)
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.0.overlaps(&other.0)
    }
}

struct Pair(RangeIn, RangeIn);

impl FromStr for Pair {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Some((l, r)) = s.trim().split(',').collect_tuple::<(_, _)>() {
            let l: RangeIn = l.parse()?;
            let r: RangeIn = r.parse()?;
            return Ok(Self(l, r));
        }

        Err(eyre!("bad input"))
    }
}

impl Pair {
    fn supersets(&self) -> bool {
        self.0.contains_or_is_contained(&self.1)
    }

    fn overlaps(&self) -> bool {
        self.0.overlaps(&self.1)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let it = io::stdin().lines().map(|l| l?.parse::<Pair>());

    let mut supersets = 0;
    let mut overlaps = 0;

    for pair in it {
        let pair = pair?;
        supersets += pair.supersets() as u32;
        overlaps += pair.overlaps() as u32;
    }

    println!("supersets: {supersets}");
    println!("overlaps:  {overlaps}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supersets() {
        let input = "\
        2-4,6-8
        2-3,4-5
        5-7,7-9
        2-8,3-7
        6-6,4-6
        2-6,4-8";

        let counts = input
            .lines()
            .map(|l| l.parse::<Pair>().expect("expected a range"))
            .filter(Pair::supersets)
            .count();

        assert_eq!(counts, 2);
    }

    #[test]
    fn overlaps() {
        let input = "\
        2-4,6-8
        2-3,4-5
        5-7,7-9
        2-8,3-7
        6-6,4-6
        2-6,4-8";

        let counts = input
            .lines()
            .map(|l| l.parse::<Pair>().expect("expected a range"))
            .filter(Pair::overlaps)
            .count();

        assert_eq!(counts, 4);
    }
}
