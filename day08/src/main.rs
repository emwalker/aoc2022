use color_eyre::{self, eyre::eyre, Result};
use std::{
    collections::HashSet,
    fmt::Debug,
    io::{self, Read},
};

enum RangeIter {
    Forward(std::ops::Range<i32>),
    Backward(std::iter::Rev<std::ops::Range<i32>>),
}

pub enum Range {
    Forward(i32),
    Backward(i32),
}

impl Iterator for RangeIter {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RangeIter::Forward(range) => range.next(),
            RangeIter::Backward(range) => range.next(),
        }
    }
}

impl Range {
    fn range(&self) -> RangeIter {
        match self {
            Self::Forward(ub) => RangeIter::Forward(0..*ub),
            Self::Backward(ub) => RangeIter::Backward((0..*ub).rev()),
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
struct Point(i32, i32);

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({},{})", self.0, self.1))
    }
}

struct Bounds {
    i: i32,
    j: i32,
}

struct Map {
    map: Vec<Vec<i32>>,
    bounds: Bounds,
}

impl Map {
    fn from_array(map: Vec<Vec<i32>>) -> Result<Self> {
        if map.is_empty() || map[0].is_empty() {
            return Err(eyre!("map cannot be empty"));
        }
        let bounds = Bounds {
            i: map.len() as i32,
            j: map[0].len() as i32,
        };

        Ok(Self { map, bounds })
    }

    fn height_at(&self, i: i32, j: i32) -> i32 {
        if self.in_bounds(i, j) {
            return self.map[i as usize][j as usize];
        }
        0
    }

    fn in_bounds(&self, i: i32, j: i32) -> bool {
        0 <= i && i < self.bounds.i && 0 <= j && j < self.bounds.j
    }

    fn at_edge(&self, i: i32, j: i32) -> bool {
        i <= 0 || (i >= self.bounds.i - 1) || j <= 0 || (j >= self.bounds.j - 1)
    }

    #[allow(unused)]
    fn len(&self) -> usize {
        self.bounds.i as usize
    }
}

struct Task {
    map: Map,
}

impl Task {
    fn parse(lines: &[String]) -> Result<Self> {
        let map: Vec<_> = lines
            .iter()
            .map(|line| {
                line.trim()
                    .chars()
                    .map(|c| c as i32 - '0' as i32)
                    .collect::<Vec<_>>()
            })
            .collect();

        let map = Map::from_array(map)?;
        Ok(Self { map })
    }

    fn visible_trees(&self) -> usize {
        let mut visible: HashSet<Point> = HashSet::new();

        // Left to right, top to bottom
        self.add_to(
            &mut visible,
            Range::Forward(self.map.bounds.i),
            Range::Forward(self.map.bounds.j),
            Point,
        );

        // Top to bottom, left to right
        self.add_to(
            &mut visible,
            Range::Forward(self.map.bounds.j),
            Range::Forward(self.map.bounds.i),
            |u, v| Point(v, u),
        );

        // Right to left, top to bottom
        self.add_to(
            &mut visible,
            Range::Forward(self.map.bounds.i),
            Range::Backward(self.map.bounds.j),
            Point,
        );

        // Bottom to top, left to right
        self.add_to(
            &mut visible,
            Range::Forward(self.map.bounds.j),
            Range::Backward(self.map.bounds.i),
            |u, v| Point(v, u),
        );

        visible.len()
    }

    fn add_to<V>(&self, visible: &mut HashSet<Point>, urange: Range, vrange: Range, point_at: V)
    where
        V: Fn(i32, i32) -> Point,
    {
        for u in urange.range() {
            let mut vmax = -1;
            for v in vrange.range() {
                let p = point_at(u, v);
                let v = self.map.height_at(p.0, p.1);

                if v > vmax {
                    visible.insert(p);
                    vmax = v;
                }
            }
        }
    }

    fn trees_ahead(&self, height: i32, i: i32, j: i32, di: i32, dj: i32) -> i32 {
        let (i, j) = (i + di, j + dj);

        if !self.map.in_bounds(i, j) {
            return 0;
        }

        if self.map.height_at(i, j) < height {
            1 + self.trees_ahead(height, i, j, di, dj)
        } else {
            1
        }
    }

    fn scenic_score(&self, i: i32, j: i32) -> i32 {
        if self.map.at_edge(i, j) {
            return 0;
        }

        let height = self.map.height_at(i, j);
        [
            // Look to the right
            self.trees_ahead(height, i, j, 0, 1),
            // Look down
            self.trees_ahead(height, i, j, 1, 0),
            // Look to the left
            self.trees_ahead(height, i, j, 0, -1),
            // Look up
            self.trees_ahead(height, i, j, -1, 0),
        ]
        .iter()
        .product()
    }

    // Is there an algorithm with less time complexity?
    fn best_scenic_score(&self) -> i32 {
        let mut max = 0;

        for i in 0..self.map.bounds.i {
            for j in 0..self.map.bounds.j {
                let score = self.scenic_score(i, j);
                if score > max {
                    max = score;
                }
            }
        }

        max
    }
}

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let lines: Vec<_> = input.lines().map(str::to_owned).collect();

    let task = Task::parse(&lines)?;
    println!("visible trees: {}", task.visible_trees());
    println!("best scenic score: {}", task.best_scenic_score());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> &'static str {
        "30373
         25512
         65332
         33549
         35390"
    }

    fn task() -> Task {
        let lines: Vec<_> = input().lines().map(str::to_string).collect();
        Task::parse(&lines).unwrap()
    }

    #[test]
    fn visible_trees() {
        let task = task();
        assert_eq!(task.map.len(), 5);
        assert_eq!(task.visible_trees(), 21);
    }

    #[test]
    fn scenic_score() {
        let task = task();

        // Trees at the edge of the map have a score of 0
        assert_eq!(task.scenic_score(0, 0), 0);
        assert_eq!(task.scenic_score(0, 4), 0);
        assert_eq!(task.scenic_score(4, 0), 0);
        assert_eq!(task.scenic_score(4, 4), 0);

        // Trees in the interior have a nonzero score
        assert_eq!(task.scenic_score(1, 2), 4);
        assert_eq!(task.scenic_score(3, 2), 8);

        assert_eq!(task.best_scenic_score(), 8);
    }
}
