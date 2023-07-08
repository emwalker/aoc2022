use color_eyre::{eyre::eyre, Report};
use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::{all_consuming, map},
    multi::fold_many1,
    sequence::tuple,
    Finish, IResult,
};
use std::str::FromStr;

pub mod naive;

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

#[derive(Debug)]
struct OreRobotInput {
    ore: Int,
}

#[derive(Debug)]
struct ClayRobotInput {
    ore: Int,
}

struct ObsidianRobotInput {
    ore: Int,
    clay: Int,
}

impl std::fmt::Debug for ObsidianRobotInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(ore: {}, clay: {})", self.ore, self.clay)
    }
}

struct GeodeRobotInput {
    ore: Int,
    obsidian: Int,
}

impl std::fmt::Debug for GeodeRobotInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(ore: {}, obsidian: {})", self.ore, self.obsidian)
    }
}

struct Blueprint {
    id: Int,
    ore_robot: OreRobotInput,
    clay_robot: ClayRobotInput,
    obsidian_robot: ObsidianRobotInput,
    geode_robot: GeodeRobotInput,
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

fn parse_ore(i: &str) -> IResult<&str, OreRobotInput> {
    map(
        tuple((
            tag("Each ore robot costs "),
            nom::character::complete::u8,
            tag(" ore."),
            multispace1,
        )),
        |(_, ore, _, _)| OreRobotInput { ore: ore as _ },
    )(i)
}

fn parse_clay(i: &str) -> IResult<&str, ClayRobotInput> {
    map(
        tuple((
            tag("Each clay robot costs "),
            nom::character::complete::u8,
            tag(" ore."),
            multispace1,
        )),
        |(_, ore, _, _)| ClayRobotInput { ore: ore as _ },
    )(i)
}

fn parse_obsidian(i: &str) -> IResult<&str, ObsidianRobotInput> {
    map(
        tuple((
            tag("Each obsidian robot costs "),
            nom::character::complete::u8,
            tag(" ore and "),
            nom::character::complete::u8,
            tag(" clay."),
            multispace1,
        )),
        |(_, ore, _, clay, _, _)| ObsidianRobotInput {
            ore: ore as _,
            clay: clay as _,
        },
    )(i)
}

fn parse_geode(i: &str) -> IResult<&str, GeodeRobotInput> {
    map(
        tuple((
            tag("Each geode robot costs "),
            nom::character::complete::u8,
            tag(" ore and "),
            nom::character::complete::u8,
            tag(" obsidian."),
            multispace0,
        )),
        |(_, ore, _, obsidian, _, _)| GeodeRobotInput {
            ore: ore as _,
            obsidian: obsidian as _,
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
