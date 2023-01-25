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

pub(crate) mod part1 {
    use super::*;

    pub struct Rucksack {
        pub shared_item: char,
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

    pub fn calculate(lines: &[String]) -> Result<i32> {
        let ruckacks: Vec<Rucksack> = lines
            .iter()
            .map(|l| l.parse::<Rucksack>())
            .collect::<Result<Vec<Rucksack>>>()?;

        let mut total = 0;
        for r in ruckacks.iter() {
            total += priority(r.shared_item)?;
        }

        Ok(total)
    }
}

pub(crate) mod part2 {
    use super::*;

    pub struct Rucksack {
        pub counts: Counter<char>,
    }

    impl FromStr for Rucksack {
        type Err = Report;

        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            let s = s.trim();
            Ok(Self {
                counts: s.chars().collect::<Counter<_>>(),
            })
        }
    }

    pub struct Group {
        rucksacks: [Rucksack; 3],
    }

    impl Group {
        pub fn parse(lines: &[String]) -> Result<Self> {
            let rucksacks: [Rucksack; 3] = lines
                .iter()
                .map(|l| l.parse::<Rucksack>())
                .collect::<Result<Vec<Rucksack>>>()?
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

    pub fn calculate(lines: &[String]) -> Result<i32> {
        let mut total = 0;
        let mut group = vec![];

        for line in lines.iter() {
            group.push(line.to_owned());

            if group.len() == 3 {
                total += Group::parse(&group)?.priority()?;
                group.clear();
            }
        }

        Ok(total)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let lines = io::stdin()
        .lock()
        .lines()
        .map(|l| l.unwrap())
        .collect::<Vec<String>>();

    let total = part1::calculate(&lines)?;
    println!("part 1: {total}");

    match part2::calculate(&lines) {
        Ok(total) => println!("part 2: {total}"),
        Err(_) => println!("part 2 could not be calculated"),
    };

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

    mod part1_test {
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
                .map(|s| s.parse::<part1::Rucksack>().unwrap().shared_item)
                .collect::<Vec<char>>();
            let expected = ['p', 'L', 'P', 'v', 't', 's'];

            assert_eq!(actual, expected);
        }
    }

    mod part2_test {
        use super::*;

        #[test]
        fn from_str() {
            let lines: Vec<String> = "\
            vJrwpWtwJgWrhcsFMMfFFhFp
            jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
            PmmdzqPrVvPwwTWBwg"
                .lines()
                .map(String::from)
                .collect();

            let g = part2::Group::parse(&lines).unwrap();
            assert_eq!(g.priority().unwrap(), 18);

            let lines: Vec<String> = "\
            wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
            ttgJtRGJQctTZtZT
            CrZsJsPPZsGzwwsLwLmpwMDw"
                .lines()
                .map(String::from)
                .collect();

            let g = part2::Group::parse(&lines).unwrap();
            assert_eq!(g.priority().unwrap(), 52);
        }
    }
}
