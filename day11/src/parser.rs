use std::collections::VecDeque;

use color_eyre::{eyre::eyre, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{complete::multispace1, streaming::multispace0},
    combinator::{all_consuming, map, value},
    multi::{fold_many1, separated_list1},
    sequence::{preceded, tuple},
    Finish, IResult,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Operator {
    Add,
    Multiply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Operand {
    Old,
    Number(i32),
}

#[derive(Debug)]
pub(crate) struct Expression {
    pub operator: Operator,
    pub operand: Operand,
}

impl Expression {
    #[allow(unused)]
    pub fn new(operator: Operator, operand: Operand) -> Self {
        Self { operand, operator }
    }
}

#[derive(Debug)]
pub(crate) struct Test {
    pub divisible_by: u32,
    pub branch_true: usize,
    pub branch_false: usize,
}

impl Test {
    #[allow(unused)]
    pub fn new(divisible_by: u32, branch_true: usize, branch_false: usize) -> Self {
        Self {
            divisible_by,
            branch_false,
            branch_true,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Monkey {
    #[allow(unused)]
    pub order: usize,
    pub operation: Expression,
    pub test: Test,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct MonkeyState {
    pub items: VecDeque<i32>,
    pub count: usize,
}

impl MonkeyState {
    pub fn new(count: usize, items: Vec<i32>) -> Self {
        Self {
            items: VecDeque::from(items),
            count,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Round(pub Vec<MonkeyState>);

#[derive(Debug)]
pub(crate) struct Notes {
    pub monkeys: Vec<Monkey>,
    #[allow(unused)]
    pub first_round: Round,
}

// Monkey 0:
fn parse_order(i: &str) -> IResult<&str, usize> {
    map(
        tuple((
            tag("Monkey "),
            nom::character::complete::u32,
            tag(":"),
            multispace1,
        )),
        |(_, i, _, _): (&str, u32, &str, &str)| i as _,
    )(i)
}

// 79, 98
fn parse_item_worry_levels(i: &str) -> IResult<&str, Vec<i32>> {
    separated_list1(
        tuple((tag(","), multispace0)),
        nom::character::complete::i32,
    )(i)
}

// Starting items: 79, 98
fn parse_items(i: &str) -> IResult<&str, Vec<i32>> {
    map(
        tuple((
            tag("Starting items: "),
            parse_item_worry_levels,
            multispace1,
        )),
        |(_, ids, _)| ids,
    )(i)
}

// * +
fn parse_operator(i: &str) -> IResult<&str, Operator> {
    alt((
        value(Operator::Multiply, tag("*")),
        value(Operator::Add, tag("+")),
    ))(i)
}

fn parse_number(i: &str) -> IResult<&str, i32> {
    map(nom::character::complete::i32, |n| n as _)(i)
}

fn parse_operand(i: &str) -> IResult<&str, Operand> {
    alt((
        value(Operand::Old, tag("old")),
        map(parse_number, Operand::Number),
    ))(i)
}

// new = old * 19
fn parse_expression(i: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            tag("new = old "),
            parse_operator,
            multispace1,
            parse_operand,
        )),
        |(_, operator, _, operand)| Expression { operator, operand },
    )(i)
}

// Operation: new = old * 19
fn parse_operation(i: &str) -> IResult<&str, Expression> {
    map(
        tuple((tag("Operation: "), parse_expression, multispace1)),
        |(_, expression, _)| expression,
    )(i)
}

// divisible by 23
fn parse_condition(i: &str) -> IResult<&str, u32> {
    map(
        tuple((
            preceded(tag("divisible by "), nom::character::complete::u32),
            multispace1,
        )),
        |(c, _)| c,
    )(i)
}

// If true: throw to monkey 2
// If false: throw to monkey 3
fn parse_branch(i: &str) -> IResult<&str, usize> {
    map(
        tuple((
            alt((
                tag("If true: throw to monkey "),
                tag("If false: throw to monkey "),
            )),
            nom::character::complete::u32,
            multispace1,
        )),
        |(_, id, _)| id as _,
    )(i)
}

fn parse_test(i: &str) -> IResult<&str, Test> {
    type Components<'s> = (&'s str, u32, usize, usize);

    map(
        tuple((tag("Test: "), parse_condition, parse_branch, parse_branch)),
        |(_, divisible_by, branch_true, branch_false): Components| Test {
            divisible_by,
            branch_true,
            branch_false,
        },
    )(i)
}

fn parse_monkey(i: &str) -> IResult<&str, (Monkey, MonkeyState)> {
    let components = tuple((parse_order, parse_items, parse_operation, parse_test));

    map(components, |(order, items, op, test)| {
        (
            Monkey {
                order,
                operation: op,
                test,
            },
            MonkeyState::new(0, items),
        )
    })(i)
}

fn parse_notes(i: &str) -> IResult<&str, Notes> {
    map(
        fold_many1(parse_monkey, Vec::new, |mut acc, pair| {
            acc.push(pair);
            acc
        }),
        |pairs| {
            let mut monkeys = vec![];
            let mut states = vec![];

            for (monkey, state) in pairs {
                monkeys.push(monkey);
                states.push(state);
            }

            Notes {
                monkeys,
                first_round: Round(states),
            }
        },
    )(i)
}

pub(crate) fn parse(i: &str) -> Result<Notes> {
    let (_s, program) = all_consuming(parse_notes)(i)
        .finish()
        .or(Err(eyre!("failed to parse input")))?;
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monkey() {
        let input = "Monkey 0:
          Starting items: 79, 98
          Operation: new = old * 19
          Test: divisible by 23
            If true: throw to monkey 2
            If false: throw to monkey 3
        ";
        let (_, (monkey, state)) = parse_monkey(input).unwrap();

        assert_eq!(monkey.order, 0);
        assert_eq!(monkey.test.branch_true, 2);
        assert_eq!(monkey.test.branch_false, 3);

        assert_eq!(state.items, vec![79, 98]);
    }

    #[test]
    fn order() {
        let (s, order) = parse_order("Monkey 0:\n").unwrap();
        assert_eq!(order, 0);
        assert_eq!(s, "");
    }

    #[test]
    fn items() {
        let (s, items) = parse_items("Starting items: 79, 98\n").unwrap();
        assert_eq!(items, vec![79, 98]);
        assert_eq!(s, "");
    }

    #[test]
    fn operation() {
        let (s, expr) = parse_operation("Operation: new = old * 19\n").unwrap();
        assert_eq!(expr.operator, Operator::Multiply);
        assert_eq!(expr.operand, Operand::Number(19));
        assert_eq!(s, "");

        let (s, expr) = parse_operation("Operation: new = old + 2\n").unwrap();
        assert_eq!(expr.operator, Operator::Add);
        assert_eq!(expr.operand, Operand::Number(2));
        assert_eq!(s, "");

        let (s, expr) = parse_operation("Operation: new = old * old\n").unwrap();
        assert_eq!(expr.operator, Operator::Multiply);
        assert_eq!(expr.operand, Operand::Old);
        assert_eq!(s, "");
    }

    #[test]
    fn test() {
        let input = "Test: divisible by 23
          If true: throw to monkey 2
          If false: throw to monkey 3
        ";

        let (s, test) = parse_test(input).unwrap();
        assert_eq!(test.divisible_by, 23);
        assert_eq!(test.branch_true, 2);
        assert_eq!(test.branch_false, 3);
        assert_eq!(s, "");
    }

    #[test]
    fn condition() {
        let (s, cond) = parse_condition("divisible by 23\n").unwrap();
        assert_eq!(cond, 23);
        assert_eq!(s, "");
    }

    #[test]
    fn branch() {
        let (s, id) = parse_branch("If true: throw to monkey 1\n").unwrap();
        assert_eq!(id, 1);
        assert_eq!(s, "");
    }

    #[test]
    fn parsing() {
        let input = include_str!("../data/example.txt");
        let notes = parse(input).unwrap();
        assert_eq!(notes.monkeys.len(), 4);
    }
}
