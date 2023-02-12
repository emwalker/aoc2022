use std::io::{self, Read};

use color_eyre::{self, Result};

mod parser;
use parser::Signal;

struct Task {
    signal: Signal,
}

impl Task {
    fn parse(input: &str) -> Result<Self> {
        let signal = parser::parse(input)?;
        Ok(Self { signal })
    }

    fn part1(&self) -> usize {
        self.signal
            .0
            .iter()
            .enumerate()
            .map(|(i, pair)| (i + 1) * (pair.is_sorted() as usize))
            .sum()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let task = Task::parse(&input)?;

    println!("sorted pair score: {}", task.part1());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> &'static str {
        "[1,1,3,1,1]
         [1,1,5,1,1]

         [[1],[2,3,4]]
         [[1],4]

         [9]
         [[8,7,6]]

         [[4,4],4,4]
         [[4,4],4,4,4]

         [7,7,7,7]
         [7,7,7]

         []
         [3]

         [[[]]]
         [[]]

         [1,[2,[3,[4,[5,6,7]]]],8,9]
         [1,[2,[3,[4,[5,6,0]]]],8,9]"
    }

    fn task(input: &str) -> Task {
        Task::parse(input).unwrap()
    }

    #[test]
    fn part1() {
        let task = task(input());
        assert_eq!(task.part1(), 13);
    }

    #[test]
    fn part1_given_input() {
        let task = task(include_str!("../data/input.txt"));
        assert_eq!(task.part1(), 5675);
    }
}
