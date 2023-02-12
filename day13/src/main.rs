use color_eyre::{self, Result};
use std::io::{self, Read};

mod parser;
use itertools::Itertools;
use parser::{Packet, Signal};

struct Task {
    signal: Signal,
}

impl Task {
    fn parse(input: &str) -> Result<Self> {
        let signal = parser::parse(input)?;
        Ok(Self { signal })
    }

    fn sorted_pair_score(&self) -> usize {
        self.signal
            .0
            .iter()
            .enumerate()
            .map(|(i, pair)| (i + 1) * (pair.is_sorted() as usize))
            .sum()
    }

    fn decoder_key_indexes(&self) -> impl Iterator<Item = usize> + '_ {
        self.signal
            .iter()
            .chain(Packet::dividers().iter())
            .sorted()
            .enumerate()
            .filter(|(_i, p)| p.is_divider())
            .map(|(i, _p)| (i + 1))
    }

    fn decoder_key(&self) -> usize {
        self.decoder_key_indexes().product()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let task = Task::parse(&input)?;

    println!("sorted pair score: {}", task.sorted_pair_score());
    println!("decoder key: {}", task.decoder_key());

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
    fn sorted_pair_score() {
        let task = task(input());
        assert_eq!(task.sorted_pair_score(), 13);
    }

    #[test]
    fn part1_given_input() {
        let task = task(include_str!("../data/input.txt"));
        assert_eq!(task.sorted_pair_score(), 5675);
    }

    #[test]
    fn decoder_key() {
        let task = task(input());
        assert_eq!(task.decoder_key_indexes().collect_vec(), vec![10, 14]);
        assert_eq!(task.decoder_key(), 140);
    }
}
