use color_eyre::{self, Result};
use std::io;

#[derive(Debug)]
struct Elf {
    pub calories: Vec<u64>,
}

impl Elf {
    fn new() -> Self {
        Self { calories: vec![] }
    }

    fn total_calories(&self) -> u64 {
        self.calories.iter().sum()
    }
}

struct Runner {
    _input: Vec<String>,
    pub elves: Vec<Elf>,
}

impl Runner {
    fn parse(input: Vec<String>) -> Result<Self> {
        let mut elves = vec![Elf::new()];

        for line in input.iter() {
            if line.is_empty() {
                elves.push(Elf::new());
                continue;
            }

            let calorie = line.parse::<u64>()?;
            if let Some(elf) = elves.last_mut() {
                elf.calories.push(calorie);
            }
        }

        Ok(Self {
            _input: input,
            elves,
        })
    }

    fn run(&self) -> u64 {
        self.elves
            .iter()
            .map(|e| e.total_calories())
            .max()
            .unwrap_or_default()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut lines: Vec<String> = vec![];

    for line in io::stdin().lines() {
        lines.push(line?);
    }

    let runner = Runner::parse(lines)?;
    let max = runner.run();

    println!("max calories: {}", max);

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
        assert_eq!(runner.run(), 24000);
    }
}
