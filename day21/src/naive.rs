use crate::{Input, Int, Step};
use color_eyre::Result;
use std::collections::HashMap;

const HUMAN: &str = "humn";

struct Trial {
    humn: Int,
    cache: HashMap<String, Option<Int>>,
    steps: HashMap<String, Step>,
}

impl Trial {
    fn dfs(&mut self, step_name: &str) -> Option<Int> {
        if step_name == HUMAN {
            return Some(self.humn);
        }

        if let Some(&val) = self.cache.get(step_name) {
            return val;
        }

        let Some(step) = self.steps.get(step_name) else {
            panic!("unknown step: {step_name}");
        };

        let val = match step.to_owned() {
            Step::Shout(ans) => Some(ans),
            Step::Mul(lhs, rhs) => Some(self.dfs(&lhs)? * self.dfs(&rhs)?),
            Step::Sub(lhs, rhs) => Some(self.dfs(&lhs)? - self.dfs(&rhs)?),

            Step::Add(lhs, rhs) => {
                let ans = if step_name == "root" {
                    self.dfs(&lhs)? - self.dfs(&rhs)?
                } else {
                    self.dfs(&lhs)? + self.dfs(&rhs)?
                };
                Some(ans)
            }

            Step::Div(lhs, rhs) => {
                let l = self.dfs(&lhs)?;
                let r = self.dfs(&rhs)?;
                let (val, rem) = (l / r, l % r);
                if rem == 0 {
                    Some(val)
                } else {
                    None
                }
            }
        };

        self.cache.insert(step_name.into(), val);
        val
    }

    fn eval(&mut self) -> Option<Int> {
        self.dfs("root")
    }
}

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

            let ans = match input.0.get(step_name).unwrap() {
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

    pub fn part2(&self) -> Option<Int> {
        let mut i = 0;
        let mut lo = self.eval(0, 1).expect("lo");
        let mut hi = self.eval(1, 1).expect("hi");
        dbg!(lo, hi);

        while lo.1.signum() == hi.1.signum() {
            lo = hi;
            hi = self.eval(lo.0 * 2, 1)?;
        }

        while i < 100 {
            let m = (lo.0 + hi.0) / 2;
            let mid = self.eval(m, 1)?;

            if mid.1.signum() == lo.1.signum() {
                lo = mid;
            } else {
                hi = mid;
            }

            if lo.1 == 0 {
                return Some(lo.0);
            }

            if hi.1 == 0 {
                return Some(hi.0);
            }

            i += 1;
        }

        None
    }

    fn eval(&self, mut k: Int, dk: Int) -> Option<(Int, Int)> {
        let mut i = 0;

        while i < 20 {
            let val = self.trial(k);

            if let Some(val) = val {
                return Some((k, val));
            }

            k += dk;
            i += 1;
        }

        None
    }

    fn trial(&self, humn: Int) -> Option<Int> {
        Trial {
            steps: self.input.0.clone(),
            cache: HashMap::new(),
            humn,
        }
        .eval()
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
    fn part2() {
        let task = parse(EXAMPLE).unwrap();
        assert_eq!(task.part2().unwrap(), 301);
    }

    #[test]
    fn input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();

        assert_eq!(task.part1(), 43_699_799_094_202);
        assert_eq!(task.part2().unwrap(), 3_375_719_472_770);
    }
}
