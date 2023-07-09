// https://github.com/Crazytieguy/advent-of-code/blob/master/2022/src/bin/day19/main.rs
use crate::{Blueprint, Input, Int, Resources, ONE_CLAY, ONE_OBSIDIAN, ONE_ORE};
use color_eyre::Result;

#[derive(Clone, Copy, Default, Debug)]
struct State {
    minutes_remaining: Int,
    resources: Resources,
    resources_rate: Resources,
}

impl State {
    fn new(minutes_remaining: Int) -> Self {
        Self {
            minutes_remaining,
            resources: Default::default(),
            resources_rate: ONE_ORE,
        }
    }

    fn choose_robot(self, cost: Resources, robot: Resources) -> Option<Self> {
        (1..self.minutes_remaining).rev().zip(0..).find_map(
            |(minutes_remaining, minutes_passed)| {
                let resources = self.resources + self.resources_rate * minutes_passed;
                resources.checked_sub(cost).map(|resources| Self {
                    minutes_remaining,
                    resources: resources + self.resources_rate,
                    resources_rate: self.resources_rate + robot,
                })
            },
        )
    }

    // Taking the current state as a starting point, enumerate the possible states that can follow.
    // Q: are the states limited to the next minute, or are they all states from the next minute
    // on?
    fn branch(self, blueprint: &Blueprint) -> impl Iterator<Item = Self> + '_ {
        // Q: How are these conditionals determined?
        let max_ore = blueprint.max_ore_cost();
        let allow_ore_robot = self.resources_rate.ore < max_ore;
        let allow_clay_robot = self.resources_rate.clay < blueprint.obsidian_robot.clay;
        let allow_obsidian_robot = self.resources_rate.obsidian < blueprint.geode_robot.obsidian
            && self.resources_rate.clay > 0;
        let allow_geode_robot = self.resources_rate.obsidian > 0;

        [
            allow_ore_robot.then(|| self.choose_robot(blueprint.ore_robot, ONE_ORE)),
            allow_clay_robot.then(|| self.choose_robot(blueprint.clay_robot, ONE_CLAY)),
            allow_obsidian_robot.then(|| self.choose_robot(blueprint.obsidian_robot, ONE_OBSIDIAN)),
            allow_geode_robot.then(|| {
                // Why are we passing in Default::default() here instead of ONE_GEODE?
                self.choose_robot(blueprint.geode_robot, Default::default())
                    .map(|state| Self {
                        resources: Resources {
                            // Q: Why are we assuming we'll get minutes_remaining geodes?
                            geode: state.resources.geode + state.minutes_remaining,
                            ..state.resources
                        },
                        ..state
                    })
            }),
        ]
        .into_iter()
        .flatten()
        .flatten()
    }

    // Something we know is an upper bound for the geode rate in all cases.  Used to trim the
    // state space to a subset of possible branches without excluding the branch that has the actual
    // maximum.  In this case, we assume that we have unlimited ore and clay and prefer building
    // geode robots when possible.
    //
    // The purpose of this method is to keep the run time within fast bounds.  But we have to be
    // careful not to exclude the answer.
    fn bound(&self, blueprint: &Blueprint) -> Int {
        let need = &blueprint.geode_robot;

        let (_, _, geodes) = (0..self.minutes_remaining).rev().fold(
            (
                self.resources.obsidian,
                self.resources_rate.obsidian,
                self.resources.geode,
            ),
            |(obsidian, obsidian_rate, geode), minutes_remaining| {
                if obsidian >= need.obsidian {
                    // We can build a geode robot
                    (
                        obsidian + obsidian_rate - need.obsidian,
                        obsidian_rate,
                        geode.saturating_add(minutes_remaining),
                    )
                } else {
                    // We can't build a geode robot yet; collect one obsidian robot
                    // Q: Why are we assuming we can build an obsidian rotot?
                    (obsidian + obsidian_rate, obsidian_rate + 1, geode)
                }
            },
        );

        geodes
    }
}

fn branch_and_bound(blueprint: &Blueprint, state: State, ans: &mut Int) {
    *ans = state.resources.geode.max(*ans);
    for state in state.branch(blueprint) {
        if state.bound(blueprint) > *ans {
            branch_and_bound(blueprint, state, ans);
        }
    }
}

pub struct Task {
    input: Input,
}

impl Task {
    pub fn total_quality_level(&self) -> Int {
        self.blueprints()
            .iter()
            .map(|blueprint| {
                let mut ans = 0;
                branch_and_bound(blueprint, State::new(24), &mut ans);
                blueprint.id * ans
            })
            .sum()
    }

    pub fn first_three(&self) -> Int {
        self.blueprints()
            .iter()
            .take(3)
            .map(|blueprint| {
                let mut ans = 0;
                branch_and_bound(blueprint, State::new(32), &mut ans);
                ans
            })
            .product()
    }

    fn blueprints(&self) -> &Vec<Blueprint> {
        &self.input.0
    }
}

pub fn parse(input: &str) -> Result<Task> {
    let input = input.parse::<Input>()?;
    Ok(Task { input })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1() {
        let task = parse(crate::EXAMPLE).unwrap();
        assert_eq!(task.total_quality_level(), 33);
    }

    #[test]
    fn part2() {
        let task = parse(crate::EXAMPLE).unwrap();
        assert_eq!(task.first_three(), 3472);
    }

    #[test]
    fn with_input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.total_quality_level(), 1150);
        assert_eq!(task.first_three(), 37367);
    }
}
