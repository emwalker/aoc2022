use color_eyre::{self, eyre::eyre, Report, Result};
use counter::Counter;
use std::{
    io::{self, BufRead},
    str::FromStr,
};

fn priority(c: char) -> Result<i32> {
    match c {
        'a'..='z' => Ok(c as i32 - 'a' as i32 + 1),
        'A'..='Z' => Ok(c as i32 - 'A' as i32 + 26 + 1),
        _ => Err(eyre!("unexpected value")),
    }
}

#[derive(Clone, Debug)]
struct Rucksack {
    pub shared_item: char,
    pub counts: Counter<char>,
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

        let counts = s.chars().collect::<Counter<_>>();

        if let Some(&shared_item) = shared.keys().next() {
            return Ok(Self {
                shared_item,
                counts,
            });
        }

        Err(eyre!("no item found more than once"))
    }
}

pub struct Group {
    rucksacks: [Rucksack; 3],
}

impl Group {
    fn new(rucksacks: &[Rucksack]) -> Result<Self> {
        let rucksacks: [Rucksack; 3] = rucksacks
            .to_vec()
            .try_into()
            .map_err(|_err| eyre!("wrong number of lines"))?;
        Ok(Self { rucksacks })
    }

    fn badge(&self) -> Result<char> {
        let c1 = self.rucksacks[0].counts.clone();
        let c2 = self.rucksacks[1].counts.clone();
        let c3 = self.rucksacks[2].counts.clone();
        let shared = c1 & c2 & c3;

        if let Some(&badge) = shared.keys().next() {
            return Ok(badge);
        }

        Err(eyre!("no badge found"))
    }

    pub fn priority(&self) -> Result<i32> {
        if let Ok(b) = self.badge() {
            return priority(b);
        }

        Err(eyre!("no badge"))
    }
}

struct Calculations {
    part1: i32,
    part2: i32,
}

impl Calculations {
    fn new(lines: &[String]) -> Result<Self> {
        let mut group = vec![];
        let mut part1 = 0;
        let mut part2 = 0;

        for line in lines.iter() {
            let r = line.parse::<Rucksack>()?;
            part1 += priority(r.shared_item)?;

            group.push(r);

            if group.len() == 3 {
                part2 += Group::new(&group)?.priority()?;
                group.clear();
            }
        }

        Ok(Calculations { part1, part2 })
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let lines = io::stdin()
        .lock()
        .lines()
        .map(|l| l.unwrap())
        .collect::<Vec<String>>();

    let calcs = Calculations::new(&lines)?;
    println!("part 1: {}", calcs.part1);
    println!("part 1: {}", calcs.part2);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_value() {
        assert_eq!(priority('a').unwrap(), 1);
        assert_eq!(priority('z').unwrap(), 26);
        assert_eq!(priority('A').unwrap(), 27);
        assert_eq!(priority('Z').unwrap(), 52);
    }

    #[test]
    fn rucksack_from_str() {
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
    fn group_from_rucksacs() {
        let rucksacks = "\
            vJrwpWtwJgWrhcsFMMfFFhFp
            jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
            PmmdzqPrVvPwwTWBwg"
            .lines()
            .map(|l| l.parse::<Rucksack>())
            .collect::<Result<Vec<Rucksack>>>()
            .unwrap();

        let g = Group::new(&rucksacks).unwrap();
        assert_eq!(g.priority().unwrap(), 18);

        let rucksacks = "\
            wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
            ttgJtRGJQctTZtZT
            CrZsJsPPZsGzwwsLwLmpwMDw"
            .lines()
            .map(|l| l.parse::<Rucksack>())
            .collect::<Result<Vec<Rucksack>>>()
            .unwrap();

        let g = Group::new(&rucksacks).unwrap();
        assert_eq!(g.priority().unwrap(), 52);
    }
}
