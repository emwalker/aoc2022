use color_eyre::{eyre::eyre, Report};
use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::{all_consuming, map},
    multi::fold_many1,
    sequence::tuple,
    Finish, IResult,
};
use std::{
    ops::{Add, Mul},
    str::FromStr,
};

pub mod branch1;

pub type Int = u16;

pub const EXAMPLE: &str = "\
Blueprint 1: \
    Each ore robot costs 4 ore. \
    Each clay robot costs 2 ore. \
    Each obsidian robot costs 3 ore and 14 clay. \
    Each geode robot costs 2 ore and 7 obsidian.

Blueprint 2: \
    Each ore robot costs 2 ore. \
    Each clay robot costs 3 ore. \
    Each obsidian robot costs 3 ore and 8 clay. \
    Each geode robot costs 3 ore and 12 obsidian.";

#[derive(Clone, Copy, Debug, Default)]
struct Resources {
    ore: Int,
    clay: Int,
    obsidian: Int,
    geode: Int,
}

const ONE_ORE: Resources = Resources {
    ore: 1,
    clay: 0,
    obsidian: 0,
    geode: 0,
};

const ONE_CLAY: Resources = Resources {
    ore: 0,
    clay: 1,
    obsidian: 0,
    geode: 0,
};

const ONE_OBSIDIAN: Resources = Resources {
    ore: 0,
    clay: 0,
    obsidian: 1,
    geode: 0,
};

impl Mul<Int> for Resources {
    type Output = Self;

    fn mul(self, rhs: Int) -> Self::Output {
        Self {
            ore: self.ore * rhs,
            clay: self.clay * rhs,
            obsidian: self.obsidian * rhs,
            geode: self.geode * rhs,
        }
    }
}

impl Add for Resources {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            ore: self.ore + other.ore,
            clay: self.clay + other.clay,
            obsidian: self.obsidian + other.obsidian,
            geode: self.geode + other.geode,
        }
    }
}

impl Resources {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        Some(Self {
            ore: self.ore.checked_sub(rhs.ore)?,
            clay: self.clay.checked_sub(rhs.clay)?,
            obsidian: self.obsidian.checked_sub(rhs.obsidian)?,
            geode: self.geode.checked_sub(rhs.geode)?,
        })
    }
}

struct Blueprint {
    id: Int,
    ore_robot: Resources,
    clay_robot: Resources,
    obsidian_robot: Resources,
    geode_robot: Resources,
}

impl std::fmt::Debug for Blueprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Blueprint {{ id: {}, ore: {:?}, clay: {:?}, obsidian: {:?} geode: {:?} }}",
            self.id, self.ore_robot.ore, self.clay_robot.ore, self.obsidian_robot, self.geode_robot
        )
    }
}

impl Blueprint {
    fn max_ore_cost(&self) -> Int {
        self.clay_robot
            .ore
            .max(self.obsidian_robot.ore)
            .max(self.geode_robot.ore)
    }
}

fn parse_id(i: &str) -> IResult<&str, Int> {
    map(
        tuple((
            tag("Blueprint "),
            nom::character::complete::u8,
            tag(":"),
            multispace1,
        )),
        |(_, id, _, _)| id as _,
    )(i)
}

fn parse_ore(i: &str) -> IResult<&str, Resources> {
    map(
        tuple((
            tag("Each ore robot costs "),
            nom::character::complete::u8,
            tag(" ore."),
            multispace1,
        )),
        |(_, ore, _, _)| Resources {
            ore: ore as _,
            ..Default::default()
        },
    )(i)
}

fn parse_clay(i: &str) -> IResult<&str, Resources> {
    map(
        tuple((
            tag("Each clay robot costs "),
            nom::character::complete::u8,
            tag(" ore."),
            multispace1,
        )),
        |(_, ore, _, _)| Resources {
            ore: ore as _,
            ..Default::default()
        },
    )(i)
}

fn parse_obsidian(i: &str) -> IResult<&str, Resources> {
    map(
        tuple((
            tag("Each obsidian robot costs "),
            nom::character::complete::u8,
            tag(" ore and "),
            nom::character::complete::u8,
            tag(" clay."),
            multispace1,
        )),
        |(_, ore, _, clay, _, _)| Resources {
            ore: ore as _,
            clay: clay as _,
            ..Default::default()
        },
    )(i)
}

fn parse_geode(i: &str) -> IResult<&str, Resources> {
    map(
        tuple((
            tag("Each geode robot costs "),
            nom::character::complete::u8,
            tag(" ore and "),
            nom::character::complete::u8,
            tag(" obsidian."),
            multispace0,
        )),
        |(_, ore, _, obsidian, _, _)| Resources {
            ore: ore as _,
            obsidian: obsidian as _,
            ..Default::default()
        },
    )(i)
}

fn parse_blueprint(i: &str) -> IResult<&str, Blueprint> {
    map(
        tuple((parse_id, parse_ore, parse_clay, parse_obsidian, parse_geode)),
        |(id, ore, clay, obsidian, geode)| Blueprint {
            id,
            ore_robot: ore,
            clay_robot: clay,
            obsidian_robot: obsidian,
            geode_robot: geode,
        },
    )(i)
}

#[derive(Debug)]
struct Input(Vec<Blueprint>);

impl FromStr for Input {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (remainder, blueprints) = all_consuming(fold_many1(
            parse_blueprint,
            Vec::new,
            |mut acc, blueprint| {
                acc.push(blueprint);
                acc
            },
        ))(s)
        .finish()
        .or(Err(eyre!("failed to parse input")))?;
        assert!(remainder.trim().is_empty());

        Ok(Input(blueprints))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input() {
        let input = EXAMPLE.parse::<Input>().unwrap();
        assert_eq!(input.0.len(), 2);

        let (b1, b2) = (&input.0[0], &input.0[1]);

        assert_eq!(b1.id, 1);
        assert_eq!(b1.ore_robot.ore, 4);
        assert_eq!(b1.clay_robot.ore, 2);
        assert_eq!(b1.obsidian_robot.ore, 3);
        assert_eq!(b1.obsidian_robot.clay, 14);
        assert_eq!(b1.geode_robot.ore, 2);
        assert_eq!(b1.geode_robot.obsidian, 7);

        assert_eq!(b2.id, 2);
        assert_eq!(b2.ore_robot.ore, 2);
        assert_eq!(b2.clay_robot.ore, 3);
        assert_eq!(b2.obsidian_robot.ore, 3);
        assert_eq!(b2.obsidian_robot.clay, 8);
        assert_eq!(b2.geode_robot.ore, 3);
        assert_eq!(b2.geode_robot.obsidian, 12);
    }
}
