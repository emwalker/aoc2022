// Following https://github.com/schubart/AdventOfCode_2022_Rust/blob/master/day20/src/lib.rs
use color_eyre::{eyre::eyre, Report, Result};
use std::str::FromStr;

type Int = i64;

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
    const DECRIPTION_KEY: Int = 811589153;

    pub fn part1(&self) -> Int {
        let values = self.mix_values(1, 1);
        self.decode(&values)
    }

    pub fn part2(&self) -> Int {
        let values = self.mix_values(Self::DECRIPTION_KEY, 10);
        self.decode(&values)
    }

    fn decode(&self, values: &Vec<Int>) -> Int {
        let n = values.len();
        let i_zero = values.iter().position(|&v| v == 0).expect("zero value");
        values[(1000 + i_zero) % n] + values[(2000 + i_zero) % n] + values[(3000 + i_zero) % n]
    }

    fn mix_values(&self, key: Int, rounds: usize) -> Vec<Int> {
        let mut numbers = self
            .input
            .0
            .iter()
            .map(|v| v * key)
            .enumerate()
            .collect::<Vec<_>>();

        // We use modulo arithmetic with n-1 in this case, apparently because we're working with
        // a circular buffer.
        // Q: Why are we using n-1? A: According to the link at the top of the file, it's due to
        // the problem statement: moving an element by (n - 1) places in a list of length n leaves
        // list unchanged.
        assert!(!numbers.is_empty());
        let n = numbers.len();

        for _ in 0..rounds {
            // O(n) * O(n) -> O(n**2)
            for i in 0..numbers.len() {
                // O(n)
                let curr_i = numbers
                    .iter()
                    .position(|n| n.0 == i)
                    .expect("index exists in array");

                // In Rust, the % operator provides the remainder rather than the modulo.  Here we
                // want a positive value when (values[i] + current_i) is negative, which is what
                // rem_euclid gives us. https://stackoverflow.com/q/31210357/61048
                let next_i = (numbers[curr_i].1 + curr_i as Int).rem_euclid(n as Int - 1);

                // Both O(n)
                let tmp = numbers.remove(curr_i);
                numbers.insert(next_i as usize, tmp);
            }
        }

        numbers.into_iter().map(|(_, v)| v).collect::<Vec<_>>()
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
        assert_eq!(task.part1(), 3);
    }

    #[test]
    fn part2() {
        let task = parse(EXAMPLE).unwrap();
        assert_eq!(
            task.mix_values(Task::DECRIPTION_KEY, 10),
            &[
                0,
                -2434767459,
                1623178306,
                3246356612,
                -1623178306,
                2434767459,
                811589153
            ]
        );
        assert_eq!(task.part2(), 1623178306);
    }

    #[test]
    fn with_input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.part1(), 14526);
    }
}
