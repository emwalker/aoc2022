use color_eyre::{self, Result};
use std::{
    collections::BTreeSet,
    io,
    iter::{Rev, Take},
};

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Elf {
    pub calories: u64,
}

impl Elf {
    fn new() -> Self {
        Self { calories: 0 }
    }

    fn calories(&self) -> u64 {
        self.calories
    }
}

struct Runner {
    _input: Vec<String>,
    pub elves: BTreeSet<Elf>,
}

impl Runner {
    fn parse(input: Vec<String>) -> Result<Self> {
        let mut elves = BTreeSet::new();
        let mut current = Some(Elf::new());

        for line in input.iter() {
            if line.is_empty() {
                if let Some(elf) = current.take() {
                    elves.insert(elf);
                }
                current = Some(Elf::new());
                continue;
            }

            let calories = line.parse::<u64>()?;
            if let Some(elf) = &mut current {
                elf.calories += calories;
            }
        }

        if let Some(elf) = current.take() {
            elves.insert(elf);
        }

        Ok(Self {
            _input: input,
            elves,
        })
    }

    fn max_calories(&self) -> u64 {
        self.elves
            .iter()
            .map(|e| e.calories())
            .max()
            .unwrap_or_default()
    }

    fn top(&self, n: usize) -> Take<Rev<std::collections::btree_set::Iter<'_, Elf>>> {
        self.elves.iter().rev().take(n)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut lines: Vec<String> = vec![];

    for line in io::stdin().lines() {
        lines.push(line?);
    }

    let runner = Runner::parse(lines)?;
    let max = runner.max_calories();
    let top_three: u64 = runner.top(3).map(|e| e.calories()).sum();

    println!("max calories: {}", max);
    println!("sum of top three: {}", top_three);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_input() {
        let input: Vec<String> = "
        1000
        2000
        3000

        4000

        5000
        6000

        7000
        8000
        9000

        10000
        "
        .trim()
        .lines()
        .map(|s| s.trim())
        .map(|s| String::from_str(s).unwrap())
        .collect::<Vec<String>>();

        let runner = Runner::parse(input).unwrap();

        assert_eq!(runner.elves.len(), 5);
        assert_eq!(runner.max_calories(), 24000);
        assert_eq!(runner.top(3).map(|e| e.calories()).sum::<u64>(), 45000);
    }
}
