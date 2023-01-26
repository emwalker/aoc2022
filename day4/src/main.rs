use std::io;

use color_eyre::{self, Report, Result};

mod ranges {
    use super::*;
    use color_eyre::eyre::eyre;
    use itertools::Itertools;
    use std::ops;
    use std::str::FromStr;

    struct Range(ops::Range<u32>);

    impl FromStr for Range {
        type Err = Report;

        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            if let Some((start, end)) = s.split('-').tuples().next() {
                let start: u32 = start.parse()?;
                let end: u32 = end.parse()?;
                return Ok(Self(std::ops::Range { start, end }));
            }

            Err(eyre!("bad input"))
        }
    }

    impl Range {
        fn contains(&self, other: &Self) -> bool {
            let ops::Range { start: s1, end: e1 } = self.0;
            let ops::Range { start: s2, end: e2 } = other.0;
            s1 <= s2 && e1 >= e2
        }

        fn overlaps(&self, other: &Self) -> bool {
            let ops::Range { start: s1, end: e1 } = self.0;
            let ops::Range { start: s2, end: e2 } = other.0;
            (s1 <= s2 && s2 <= e1) || (s2 <= s1 && s1 <= e2)
        }
    }

    pub struct Pair(Range, Range);

    impl FromStr for Pair {
        type Err = Report;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if let Some((left, right)) = s.trim().split(',').tuples().next() {
                let left: Range = left.parse()?;
                let right: Range = right.parse()?;
                return Ok(Self(left, right));
            }

            Err(eyre!("bad input"))
        }
    }

    impl Pair {
        pub fn superset_exists(&self) -> bool {
            self.0.contains(&self.1) || self.1.contains(&self.0)
        }

        pub fn is_overlapping(&self) -> bool {
            self.0.overlaps(&self.1)
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let it = io::stdin().lines().map(|l| l?.parse::<ranges::Pair>());

    let mut supersets = 0;
    let mut overlaps = 0;

    for pair in it {
        let pair = pair?;
        supersets += pair.superset_exists() as u32;
        overlaps += pair.is_overlapping() as u32;
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

        let supersets: usize = input
            .lines()
            .map(|l| -> Result<ranges::Pair> { l.parse::<ranges::Pair>() })
            .collect::<Result<Vec<_>>>()
            .unwrap()
            .into_iter()
            .filter(ranges::Pair::superset_exists)
            .count();

        assert_eq!(supersets, 2);
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

        let overlaps: usize = input
            .lines()
            .map(|l| -> Result<ranges::Pair> { l.parse::<ranges::Pair>() })
            .collect::<Result<Vec<_>>>()
            .unwrap()
            .into_iter()
            .filter(ranges::Pair::is_overlapping)
            .count();

        assert_eq!(overlaps, 4);
    }
}
