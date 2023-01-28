// Largely taken from https://fasterthanli.me/series/advent-of-code-2022/part-7#using-a-stack
use color_eyre::{self, Result};
use std::io::{self, Read};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Node {
    name: String,
    children: Vec<Node>,
    size: usize,
}

impl Node {
    fn total_size(&self) -> usize {
        self.size + self.children.iter().map(|n| n.total_size()).sum::<usize>()
    }

    fn is_directory(&self) -> bool {
        !self.children.is_empty()
    }

    fn subdirs(&self) -> Box<dyn Iterator<Item = &Node> + '_> {
        Box::new(
            std::iter::once(self).chain(
                self.children
                    .iter()
                    .filter(|n| n.is_directory())
                    .flat_map(|d| d.subdirs()),
            ),
        )
    }
}

mod parser {
    use super::*;
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_while1},
        combinator::{all_consuming, map},
        sequence::{preceded, tuple},
        Finish, IResult,
    };

    #[derive(Debug)]
    pub enum Line {
        Chdir(String),
        Dir(String),
        File(String, usize),
        Ls,
    }

    fn parse_identifier(i: &str) -> IResult<&str, String> {
        map(
            take_while1(|c: char| "abcdefghijklmnopqrstuvwxyz./".contains(c)),
            str::to_owned,
        )(i)
    }

    fn parse_chdir(i: &str) -> IResult<&str, Line> {
        map(preceded(tag("$ cd "), parse_identifier), |name| {
            Line::Chdir(name)
        })(i)
    }

    fn parse_dir(i: &str) -> IResult<&str, Line> {
        map(preceded(tag("dir "), parse_identifier), |name| {
            Line::Dir(name)
        })(i)
    }

    fn parse_number(i: &str) -> IResult<&str, usize> {
        map(nom::character::complete::u32, |n| n as _)(i)
    }

    fn parse_file(i: &str) -> IResult<&str, Line> {
        map(
            tuple((parse_number, tag(" "), parse_identifier)),
            |(size, _, name)| Line::File(name, size),
        )(i)
    }

    fn parse_ls(i: &str) -> IResult<&str, Line> {
        map(tag("$ ls"), |_| Line::Ls)(i)
    }

    fn parse_line(i: &str) -> IResult<&str, Option<Line>> {
        alt((
            map(parse_chdir, Some),
            map(parse_dir, Some),
            map(parse_file, Some),
            map(parse_ls, Some),
        ))(i)
    }

    pub struct Ast(pub Vec<Option<Line>>);

    impl Ast {
        pub fn finalize(self) -> Result<Node> {
            let Self(lines) = self;

            let mut stack = vec![Node {
                name: "/".into(),
                children: vec![],
                size: 0,
            }];

            for line in lines {
                match line {
                    Some(Line::Chdir(name)) => match name.as_str() {
                        "/" => {}

                        ".." => {
                            let child = stack.pop().unwrap();
                            stack.last_mut().unwrap().children.push(child);
                        }

                        _ => {
                            let node = Node {
                                name: name.to_owned(),
                                children: vec![],
                                size: 0,
                            };
                            stack.push(node);
                        }
                    },

                    Some(Line::File(name, size)) => {
                        let node = Node {
                            name: name.to_owned(),
                            children: vec![],
                            size,
                        };
                        stack.last_mut().unwrap().children.push(node);
                    }

                    _ => {}
                }
            }

            let mut root = stack.pop().unwrap();
            while let Some(mut next) = stack.pop() {
                next.children.push(root);
                root = next;
            }

            Ok(root)
        }
    }

    pub fn parse(input: &str) -> Result<Ast> {
        let lines: Vec<_> = input
            .lines()
            .map_while(|line| {
                all_consuming(parse_line)(line.trim())
                    .finish()
                    .ok()
                    .map(|(_, l)| l)
            })
            .collect();

        Ok(Ast(lines))
    }
}

struct Task(Node);

impl Task {
    fn new(node: Node) -> Self {
        Self(node)
    }

    fn part1(&self) -> usize {
        self.0
            .subdirs()
            .map(|d| d.total_size())
            .filter(|n| *n <= 100_000)
            .sum()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let root = parser::parse(&input).unwrap().finalize()?;
    let task = Task::new(root);

    println!("part 1: {}", task.part1());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{parse, Line};

    fn input<'s>() -> &'s str {
        "$ cd /
        $ ls
        dir a
        14848514 b.txt
        8504156 c.dat
        dir d
        $ cd a
        $ ls
        dir e
        29116 f
        2557 g
        62596 h.lst
        $ cd e
        $ ls
        584 i
        $ cd ..
        $ cd ..
        $ cd d
        $ ls
        4060174 j
        8033020 d.log
        5626152 d.ext
        7214296 k"
    }

    #[test]
    fn parse_ast() {
        let ast = parse(input()).unwrap().0;
        assert_eq!(ast.len(), 23);

        assert!(matches!(
            &ast[0],
            Some(Line::Chdir(name)) if name == "/"));

        assert!(matches!(&ast[1], Some(Line::Ls)));
        assert!(matches!(&ast[2], Some(Line::Dir(name)) if name == "a"));
        assert!(matches!(&ast[3], Some(Line::File(name, size))
            if name == "b.txt" && *size == 14848514 ));
        assert!(matches!(&ast[22], Some(Line::File(name, size))
            if name == "k" && *size == 7214296 ));
    }

    #[test]
    fn part1() {
        let root = parse(input()).unwrap().finalize().unwrap();
        let task = Task::new(root);
        assert_eq!(task.part1(), 95437);
    }
}
