use crate::{Axis, Input, Int, Point};
use color_eyre::Result;
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Hash, Eq, PartialEq)]
struct Key(Int, Int);

impl Point {
    fn key(&self, dim: Axis) -> Key {
        match dim {
            Axis::X => Key(self.0[1], self.0[2]),
            Axis::Y => Key(self.0[0], self.0[2]),
            Axis::Z => Key(self.0[0], self.0[1]),
        }
    }
}

#[derive(Debug)]
struct Column {
    points: Vec<Point>,
    axis: Axis,
}

impl Column {
    fn push(&mut self, p: Point) {
        self.points.push(p)
    }

    fn values(&self) -> impl Iterator<Item = Int> + '_ {
        self.points.iter().map(|p| p.0[self.axis as usize])
    }
}

// Area of the shape for a given axis
#[derive(Debug)]
struct PartialArea {
    axis: Axis,
    points: HashMap<Key, Column>,
}

impl PartialArea {
    fn new(axis: Axis) -> Self {
        Self {
            axis,
            points: HashMap::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    fn add(&mut self, p: Point) {
        let key = p.key(self.axis);

        self.points
            .entry(key)
            .or_insert(Column {
                axis: self.axis,
                points: vec![],
            })
            .push(p);
    }

    fn columns(&self) -> impl Iterator<Item = &Column> + '_ {
        self.points.values()
    }
}

#[derive(Debug)]
struct State {
    areas: [PartialArea; 3],
}

impl State {
    fn area(&self) -> Int {
        let mut ans = 0;

        if self.areas.iter().any(|a| a.is_empty()) {
            return 0;
        }

        for area in &self.areas {
            for col in area.columns() {
                let sorted = col.values().sorted().collect_vec();

                let val = sorted
                    .iter()
                    .zip(sorted[1..].iter())
                    .map(|(&m, &n)| if m < n - 1 { 2 } else { 0 })
                    .sum::<Int>();

                ans += 2 + val;
            }
        }

        ans
    }
}

pub struct Task {
    input: Input,
}

impl Task {
    pub fn surface_area(&self) -> Int {
        self.state().area()
    }

    fn state(&self) -> State {
        let mut areas = [
            PartialArea::new(Axis::X),
            PartialArea::new(Axis::Y),
            PartialArea::new(Axis::Z),
        ];

        for &p in &self.input.0 {
            for area in &mut areas {
                area.add(p);
            }
        }

        State { areas }
    }
}

pub fn parse(s: &str) -> Result<Task> {
    let input = s.parse::<Input>()?;
    Ok(Task { input })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
    2,2,2
    1,2,2
    3,2,2
    2,1,2
    2,3,2
    2,2,1
    2,2,3
    2,2,4
    2,2,6
    1,2,5
    3,2,5
    2,1,5
    2,3,5";

    #[test]
    fn part1() {
        let task = parse(EXAMPLE).unwrap();
        assert_eq!(task.surface_area(), 64);
    }

    #[test]
    fn with_input() {
        let input = include_str!("../data/input.txt");
        let task = parse(input).unwrap();
        assert_eq!(task.surface_area(), 4636);
    }
}
