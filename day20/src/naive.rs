// Following https://github.com/schubart/AdventOfCode_2022_Rust/blob/master/day20/src/lib.rs
use color_eyre::{eyre::eyre, Report, Result};
use std::str::FromStr;

type Int = i16;

struct Input(Vec<Int>);

impl FromStr for Input {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let input = s
            .trim()
            .lines()
            .map(|l| {
                l.trim()
                    .parse::<Int>()
                    .or(Err(eyre!("failed to parse integer")))
            })
            .collect::<Result<Vec<Int>>>()
            .or(Err(eyre!("failed to parse input")))?;

        Ok(Self(input))
    }
}

pub struct Task {
    input: Input,
}

impl Task {
    pub fn part1(&self) -> Int {
        let n = self.input.0.len();
        let values = self.shuffled_values();
        let i_zero = values.iter().position(|&v| v == 0).expect("zero value");
        values[(1000 + i_zero) % n] + values[(2000 + i_zero) % n] + values[(3000 + i_zero) % n]
    }

    fn shuffled_values(&self) -> Vec<Int> {
        let values = self.input.0.clone();
        let mut indexes = (0..values.len()).collect::<Vec<_>>();

        // We use modulo arithmetic with n-1 in this case, apparently because we're working with
        // a circular buffer.
        // Q: Why are we using n-1?
        assert!(!values.is_empty());
        let n = values.len() - 1;

        // O(n) * O(n) -> O(n**2)
        for (i, val) in values.iter().enumerate() {
            // O(n)
            let curr_i = indexes
                .iter()
                .position(|&k| k == i)
                .expect("index exists in array");

            // In Rust, the % operator provides the remainder rather than the modulo.  Here we want
            // a positive value when (values[i] + current_i) is negative, which is what rem_euclid
            // gives us. https://stackoverflow.com/q/31210357/61048
            let mut next_i = (val + curr_i as Int).rem_euclid(n as Int) as usize;

            // Q: Why do we have to reset to n-1 here?
            if next_i == 0 {
                next_i = n;
            }

            // Both O(n)
            let tmp = indexes.remove(curr_i);
            indexes.insert(next_i, tmp);
        }

        indexes.iter().map(|&i| values[i]).collect()
    }
}

pub fn parse(s: &str) -> Result<Task> {
    let input = s.parse::<Input>()?;
    Ok(Task { input })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
    1
    2
    -3
    3
    -2
    0
    4";

    #[test]
    fn part1() {
        let task = parse(EXAMPLE).unwrap();
        assert_eq!(task.shuffled_values(), &[1, 2, -3, 4, 0, 3, -2]);
        assert_eq!(task.part1(), 3);
    }

    #[test]
    fn with_input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.part1(), 14526);
    }
}
