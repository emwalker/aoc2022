use color_eyre::{self, Result};
use std::{
    collections::BinaryHeap,
    io::{self, Read},
};

mod parser;
use parser::{Notes, Operand, Operator, Round};

impl parser::Expression {
    fn evaluate(&self, n: i32) -> i32 {
        let v = match self.operator {
            Operator::Add => self.lhs(n) + n,
            Operator::Multiply => self.lhs(n) * n,
        };
        v / 3
    }

    fn lhs(&self, n: i32) -> i32 {
        match self.operand {
            Operand::Old => n,
            Operand::Number(v) => v,
        }
    }
}

impl parser::Test {
    fn branch(&self, n: i32) -> usize {
        let divisor = self.divisible_by as i32;
        if n % divisor == 0 {
            self.branch_true
        } else {
            self.branch_false
        }
    }
}

struct RoundIter<'n> {
    notes: &'n Notes,
    prev: Round,
}

impl<'n> Iterator for RoundIter<'n> {
    type Item = Round;

    fn next(&mut self) -> Option<Self::Item> {
        let mut states = self.prev.0.clone();

        for (i, monkey) in self.notes.monkeys.iter().enumerate() {
            while !states[i].items.is_empty() {
                if let Some(item) = states[i].items.pop_front() {
                    states[i].count += 1;
                    let new_level = monkey.operation.evaluate(item);
                    let dest = monkey.test.branch(new_level);
                    states[dest].items.push_back(new_level);
                }
            }
        }

        self.prev = Round(states);
        Some(self.prev.clone())
    }
}

impl parser::Notes {
    fn rounds(&self) -> RoundIter {
        RoundIter {
            notes: self,
            prev: self.first_round.clone(),
        }
    }
}

#[derive(Debug)]
struct Task(parser::Notes);

impl Task {
    fn parse(input: &str) -> Result<Self> {
        let notes = parser::parse(input)?;
        Ok(Self(notes))
    }

    fn monkey_business(&self) -> Option<usize> {
        let mut counts = BinaryHeap::new();
        let round = self.0.rounds().take(20).last()?;

        for state in round.0.into_iter() {
            counts.push(state.count);
        }

        let mut value = 1;

        for _ in 0..2 {
            value *= counts.pop()?;
        }

        Some(value)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = Task::parse(&input)?;
    println!(
        "monkey business: {}",
        task.monkey_business().unwrap_or_default()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use parser::{Expression, MonkeyState, Test};

    fn task() -> Task {
        let input = include_str!("../data/example.txt").to_owned();
        Task::parse(&input).unwrap()
    }

    #[test]
    fn evaluation() {
        let test = Expression::new(Operator::Multiply, Operand::Number(19));
        assert_eq!(500, test.evaluate(79));
    }

    #[test]
    fn branching() {
        let test = Test::new(23, 2, 3);
        assert_eq!(test.branch(46), 2);
        assert_eq!(test.branch(500), 3);
    }

    #[test]
    fn round() {
        let Task(notes) = task();

        assert_eq!(
            notes.monkeys.iter().map(|m| m.order).collect_vec(),
            vec![0, 1, 2, 3]
        );

        let rounds = notes.rounds().take(20).collect_vec();

        // Round 1
        let next = &rounds[0].0;
        assert_eq!(next[0], MonkeyState::new(2, vec![20, 23, 27, 26]));
        assert_eq!(
            next[1],
            MonkeyState::new(4, vec![2080, 25, 167, 207, 401, 1046])
        );
        assert_eq!(next[2], MonkeyState::new(3, vec![]));
        assert_eq!(next[3], MonkeyState::new(5, vec![]));

        // Round 2
        let next = &rounds[1].0;
        assert_eq!(next[0], MonkeyState::new(6, vec![695, 10, 71, 135, 350]));
        assert_eq!(next[1], MonkeyState::new(10, vec![43, 49, 58, 55, 362]));
        assert_eq!(next[2], MonkeyState::new(4, vec![]));
        assert_eq!(next[3], MonkeyState::new(10, vec![]));

        // Round 3
        let next = &rounds[2].0;
        assert_eq!(next[0], MonkeyState::new(11, vec![16, 18, 21, 20, 122]));
        assert_eq!(next[1], MonkeyState::new(15, vec![1468, 22, 150, 286, 739]));
        assert_eq!(next[2], MonkeyState::new(4, vec![]));
        assert_eq!(next[3], MonkeyState::new(15, vec![]));

        // Round 15
        let next = &rounds[14].0;
        assert_eq!(
            next[0],
            MonkeyState::new(73, vec![83, 44, 8, 184, 9, 20, 26, 102])
        );
        assert_eq!(next[1], MonkeyState::new(73, vec![110, 36]));
        assert_eq!(next[2], MonkeyState::new(6, vec![]));
        assert_eq!(next[3], MonkeyState::new(77, vec![]));

        // Round 20
        let next = &rounds[19].0;
        assert_eq!(next[0], MonkeyState::new(101, vec![10, 12, 14, 26, 34]));
        assert_eq!(next[1], MonkeyState::new(95, vec![245, 93, 53, 199, 115]));
        assert_eq!(next[2], MonkeyState::new(7, vec![]));
        assert_eq!(next[3], MonkeyState::new(105, vec![]));
    }

    #[test]
    fn monkey_business() {
        let task = task();
        assert_eq!(10605, task.monkey_business().unwrap());
    }
}
