// Following https://github.com/Crazytieguy/advent-of-code/blob/master/2022/src/bin/day21/main.rs
use crate::{parse_input, Input, Int, Step};
use color_eyre::Result;
use std::collections::HashMap;

const HUMAN: &str = "humn";
const ROOT: &str = "root";

#[derive(Copy, Clone)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    fn eval(&self, l: Int, r: Int) -> Int {
        match self {
            Self::Add => l + r,
            Self::Sub => l - r,
            Self::Mul => l * r,
            Self::Div => l / r,
        }
    }

    fn solve_for_left(&self, ans: Int, r: Int) -> Int {
        match self {
            Self::Add => ans - r,
            Self::Sub => ans + r,
            Self::Mul => ans / r,
            Self::Div => ans * r,
        }
    }

    fn solve_for_right(&self, ans: Int, l: Int) -> Int {
        match self {
            Self::Add => ans - l,
            Self::Sub => l - ans,
            Self::Mul => ans / l,
            Self::Div => l / ans,
        }
    }
}

#[derive(Copy, Clone)]
enum Expression<'s> {
    Shout(Int),
    Operation((&'s str, Op, &'s str)),
}

impl<'s> Expression<'s> {
    fn from(step: &'s Step) -> Self {
        match step {
            Step::Shout(v) => Expression::Shout(*v),
            Step::Add(lhs, rhs) => Expression::Operation((lhs, Op::Add, rhs)),
            Step::Sub(lhs, rhs) => Expression::Operation((lhs, Op::Sub, rhs)),
            Step::Mul(lhs, rhs) => Expression::Operation((lhs, Op::Mul, rhs)),
            Step::Div(lhs, rhs) => Expression::Operation((lhs, Op::Div, rhs)),
        }
    }
}

struct Expressions<'s>(HashMap<&'s str, Expression<'s>>);

impl<'s> Expressions<'s> {
    fn from(steps: &'s HashMap<&'s str, Step>) -> Self {
        let statements = steps
            .iter()
            .map(|(&name, step)| (name, Expression::from(step)))
            .collect::<HashMap<_, _>>();
        Self(statements)
    }

    fn fill_knowns(&mut self, knowns: &mut HashMap<&'s str, Int>, name: &'s str) -> Option<Int> {
        if name == HUMAN {
            return None;
        }

        let val = match self.0[name] {
            Expression::Shout(val) => val,

            Expression::Operation((lhs, op, rhs)) => {
                let left = self.fill_knowns(knowns, lhs);
                let right = self.fill_knowns(knowns, rhs);
                op.eval(left?, right?)
            }
        };
        knowns.insert(name, val);

        Some(val)
    }
}

pub struct Task<'s> {
    pub input: Input<'s>,
}

impl<'s> Task<'s> {
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

        dfs(ROOT, &self.input, &mut cache)
    }

    pub fn part2(&self) -> Int {
        let mut stmts = Expressions::from(&self.input.0);
        let mut knowns = HashMap::new();

        stmts.fill_knowns(&mut knowns, "root");

        // We add a "correction" to accomplish a subtraction for the root expression.  In the
        // problem statement for part 2, the root operation compares the left and right values
        // for equality.  To implement this, we subtract the right from the left and look for zero.
        // If we get a result of zero, we have the value for humn that we're looking for.  The
        // correction accomplishes this by turning the addition for the root node into a
        // subtraction.  After we've done this once, we can treat all subsequent additions as usual.
        let (mut unknown, mut ans, mut correction) = (ROOT, 0, -1);

        while unknown != HUMAN {
            let Expression::Operation((lhs, op, rhs)) = stmts.0[unknown] else {
                panic!("{unknown} not found");
            };

            (unknown, ans) = match (knowns.get(&lhs), knowns.get(&rhs)) {
                (None, Some(&r)) => (lhs, op.solve_for_left(ans, r)),
                (Some(&l), None) => (rhs, op.solve_for_right(ans, l)),
                _ => unreachable!(),
            };

            ans *= correction;
            correction = 1;
        }

        ans
    }
}

pub fn parse(s: &'static str) -> Result<Task> {
    let input = parse_input(s)?;
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
        assert_eq!(task.part2(), 301);
    }

    #[test]
    fn input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();

        assert_eq!(task.part1(), 43_699_799_094_202);
        assert_eq!(task.part2(), 3_375_719_472_770);
    }
}
