use crate::{Input, Int, Step};
use color_eyre::Result;
use std::collections::HashMap;

pub struct Task {
    pub input: Input,
}

impl Task {
    pub fn part1(&self) -> Int {
        let mut cache = HashMap::<String, Int>::new();

        fn dfs(step_name: &str, input: &Input, cache: &mut HashMap<String, Int>) -> Int {
            if let Some(&ans) = cache.get(step_name) {
                return ans;
            }

            let ans = match input.get(step_name).unwrap() {
                Step::Shout(ans) => *ans,
                Step::Add(lhs, rhs) => dfs(lhs, input, cache) + dfs(rhs, input, cache),
                Step::Mul(lhs, rhs) => dfs(lhs, input, cache) * dfs(rhs, input, cache),
                Step::Sub(lhs, rhs) => dfs(lhs, input, cache) - dfs(rhs, input, cache),
                Step::Div(lhs, rhs) => dfs(lhs, input, cache) / dfs(rhs, input, cache),
            };

            cache.insert(step_name.into(), ans);
            ans
        }

        dfs("root", &self.input, &mut cache)
    }
}

pub fn parse(s: &str) -> Result<Task> {
    let input = s.parse::<Input>()?;
    Ok(Task { input })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EXAMPLE;

    #[test]
    fn part1() {
        let task = parse(EXAMPLE).unwrap();
        assert_eq!(task.part1(), 152);
    }

    #[test]
    fn input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.part1(), 43_699_799_094_202);
    }
}
