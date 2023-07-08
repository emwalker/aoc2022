use crate::{Blueprint, Input, Int};
use color_eyre::Result;

#[derive(Clone, Debug, Default)]
struct Kinds {
    ore: Int,
    clay: Int,
    obsidian: Int,
    geode: Int,
}

#[derive(Clone, Debug)]
struct State {
    resources: Kinds,
    robots: Kinds,
}

impl State {
    fn geode_robots(&mut self, blueprint: &Blueprint) {
        let need = &blueprint.geode_robot;
        let ore_allows = self.resources.ore / need.ore;
        let obsidian_allows = self.resources.obsidian / need.obsidian;
        let build = need.ore <= self.resources.ore && need.obsidian <= self.resources.obsidian;

        let mut builder = |count: Int| {
            if build {
                self.resources.ore -= count * need.ore;
                self.resources.obsidian -= count * need.obsidian;
                self.robots.geode += count;
            }
        };

        if ore_allows <= obsidian_allows {
            builder(ore_allows);
            self.ore_robots(blueprint);
        } else {
            builder(obsidian_allows);
            self.obsidian_robots(blueprint);
        }
    }

    fn obsidian_robots(&mut self, blueprint: &Blueprint) {
        let need = &blueprint.obsidian_robot;
        let ore_allows = self.resources.ore / need.ore;
        let clay_allows = self.resources.clay / need.clay;
        let build = need.ore <= self.resources.ore && need.clay <= self.resources.clay;

        let mut builder = |count: Int| {
            if build {
                self.resources.ore -= count * need.ore;
                self.resources.clay -= count * need.clay;
                self.robots.obsidian += count;
            }
        };

        if ore_allows < clay_allows {
            builder(ore_allows);
            self.ore_robots(blueprint)
        } else {
            builder(clay_allows);
            self.clay_robots(blueprint)
        }
    }

    fn clay_robots(&mut self, blueprint: &Blueprint) {
        let need = &blueprint.clay_robot;
        let ore_allows = self.resources.ore / need.ore;

        if ore_allows > 0 {
            self.resources.ore -= ore_allows * need.ore;
            self.robots.clay += ore_allows;
        }

        self.ore_robots(blueprint)
    }

    fn ore_robots(&mut self, blueprint: &Blueprint) {
        let need = &blueprint.ore_robot;
        let count = self.resources.ore / need.ore;

        if count > 0 {
            self.resources.ore -= count * need.ore;
            self.robots.ore += count;
        }
    }
}

pub struct Task {
    input: Input,
    minutes: u8,
}

impl Task {
    pub fn total_quality_level(&self) -> Result<Int> {
        let mut ans = 0;

        for blueprint in &self.input.0 {
            let mut curr = State {
                resources: Kinds::default(),
                robots: Kinds {
                    ore: 1,
                    ..Default::default()
                },
            };

            for _ in 0..self.minutes {
                let prev = curr.clone();
                curr.geode_robots(blueprint);

                curr.resources.ore += prev.robots.ore;
                curr.resources.clay += prev.robots.clay;
                curr.resources.obsidian += prev.robots.obsidian;
                curr.resources.geode += prev.robots.geode;
            }

            ans += blueprint.id * curr.resources.geode;
        }

        Ok(ans)
    }
}

pub fn parse(input: &str) -> Result<Task> {
    let input = input.parse::<Input>()?;
    Ok(Task { input, minutes: 24 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1() {
        let task = parse(crate::EXAMPLE).unwrap();
        assert_eq!(task.total_quality_level().unwrap(), 33);
    }

    #[test]
    fn with_input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.total_quality_level().unwrap(), 823);
    }
}
