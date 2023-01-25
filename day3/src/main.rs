use color_eyre::{self, eyre::eyre, Report, Result};
use counter::Counter;
use std::{
    io::{self, BufRead},
    str::FromStr,
};

struct Rucksack {
    shared_item: char,
}

impl FromStr for Rucksack {
    type Err = Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();
        let len = s.len();

        if len % 2 != 0 {
            return Err(eyre!("an even sized string is required"));
        }

        fn count(t: &str) -> Counter<char> {
            t.chars().collect::<Counter<_>>()
        }

        let len = len / 2;
        let (left, right) = (count(&s[..len]), count(&s[len..]));
        let shared = left & right;

        if let Some(&shared_item) = shared.keys().next() {
            return Ok(Self { shared_item });
        }

        Err(eyre!("no item found more than once"))
    }
}

impl Rucksack {
    fn priority(&self) -> Result<i32> {
        match self.shared_item {
            'a'..='z' => Ok(self.shared_item as i32 - 'a' as i32 + 1),
            'A'..='Z' => Ok(self.shared_item as i32 - 'A' as i32 + 26 + 1),
            _ => Err(eyre!("unexpected value")),
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let ruckacks: Vec<Rucksack> = io::stdin()
        .lock()
        .lines()
        .map(|l| l?.parse::<Rucksack>())
        .collect::<Result<Vec<Rucksack>>>()?;

    let mut total = 0;
    for r in ruckacks.iter() {
        total += r.priority()?;
    }

    println!("{total}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let input = "\
        vJrwpWtwJgWrhcsFMMfFFhFp
        jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
        PmmdzqPrVvPwwTWBwg
        wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
        ttgJtRGJQctTZtZT
        CrZsJsPPZsGzwwsLwLmpwMDw";

        let actual = input
            .lines()
            .map(|s| s.parse::<Rucksack>().unwrap().shared_item)
            .collect::<Vec<char>>();
        let expected = ['p', 'L', 'P', 'v', 't', 's'];

        assert_eq!(actual, expected);
    }

    #[test]
    fn priority() {
        fn p(shared_item: char) -> i32 {
            Rucksack { shared_item }.priority().unwrap()
        }

        assert_eq!(p('a'), 1);
        assert_eq!(p('z'), 26);
        assert_eq!(p('A'), 27);
        assert_eq!(p('Z'), 52);
    }
}
