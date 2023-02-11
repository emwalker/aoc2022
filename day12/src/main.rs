use color_eyre::{self, eyre::eyre, Result};
use std::{
    collections::{BinaryHeap, HashSet},
    fmt::Debug,
    io::{self, Read},
};

#[derive(Debug, Eq, PartialEq)]
enum Cell {
    Start,
    Height(u8),
    End,
}

impl Cell {
    fn elevation(&self) -> u8 {
        match self {
            Cell::Start => 0,
            Cell::Height(h) => *h,
            Cell::End => 25,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Position(i32, i32);

const NEIGHBORS: &[Position] = &[
    Position(1, 0),
    Position(0, 1),
    Position(-1, 0),
    Position(0, -1),
];

impl Position {
    fn neighbors(&self) -> Vec<Position> {
        let mut neighbors = Vec::with_capacity(4);
        for Position(di, dj) in NEIGHBORS.iter() {
            let v = Position(self.0 + di, self.1 + dj);
            neighbors.push(v);
        }
        neighbors
    }
}

#[derive(Debug)]
struct Map {
    end: Position,
    grid: Vec<Vec<Cell>>,
    height: usize,
    start: Position,
    width: usize,
    lowest: Vec<Position>,
}

impl Map {
    fn parse(input: &str) -> Result<Self> {
        let mut grid = vec![];
        let mut start = None;
        let mut end = None;
        let mut lowest = vec![];

        for (i, line) in input.trim().lines().enumerate() {
            let line = line.trim();
            let mut row = Vec::with_capacity(line.len());

            for (j, c) in line.chars().enumerate() {
                let cell = match c {
                    'S' => Cell::Start,
                    'E' => Cell::End,
                    'a'..='z' => Cell::Height(c as u8 - b'a'),
                    _ => return Err(eyre!("unknown elevation: {c}")),
                };

                let pos = Position(i as i32, j as i32);

                if c == 'a' || c == 'S' {
                    lowest.push(pos);
                }

                if cell == Cell::Start {
                    if start.is_some() {
                        return Err(eyre!("start already seen"));
                    }
                    start = Some(pos);
                }

                if cell == Cell::End {
                    if end.is_some() {
                        return Err(eyre!("end already seen"));
                    }
                    end = Some(pos);
                }

                row.push(cell);
            }
            grid.push(row);
        }

        if start.is_none() || end.is_none() {
            return Err(eyre!("missing start or end"));
        }

        if grid.is_empty() || grid[0].is_empty() {
            return Err(eyre!("grid is empty"));
        }

        let height = grid.len();
        let width = grid[0].len();

        Ok(Self {
            end: end.unwrap(),
            lowest,
            grid,
            height,
            start: start.unwrap(),
            width,
        })
    }

    fn walkable_neighbors(&self, u: Position) -> impl Iterator<Item = Position> + '_ {
        let curr_elev = self.elevation(u);
        u.neighbors()
            .into_iter()
            .filter(move |v| self.can_visit(*v, curr_elev))
    }

    fn can_visit(&self, v: Position, curr_elev: u8) -> bool {
        if !self.in_bounds(v) {
            return false;
        }
        self.elevation(v) <= curr_elev + 1
    }

    fn in_bounds(&self, p: Position) -> bool {
        0 <= p.0 && p.0 < self.height as i32 && 0 <= p.1 && p.1 < self.width as i32
    }

    fn elevation(&self, p: Position) -> u8 {
        self.grid[p.0 as usize][p.1 as usize].elevation()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct Step {
    steps: i32,
    pos: Position,
}

impl Ord for Step {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.steps.cmp(&other.steps)
    }
}

impl PartialOrd for Step {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
struct Task(Map);

impl Task {
    fn parse(input: &str) -> Result<Self> {
        let map = Map::parse(input)?;
        Ok(Task(map))
    }

    // Thanks to https://github.com/NickyMeuleman/scrapyard/blob/main/advent_of_code/2022/src/day_12.rs
    fn mininium_steps(&self, u: Position) -> Option<i32> {
        let map = &self.0;

        let mut visited = HashSet::from([map.start]);
        let mut pq = BinaryHeap::from([Step { steps: 0, pos: u }]);

        while let Some(Step { steps, pos: u }) = pq.pop() {
            if u == map.end {
                return Some(-steps);
            }

            for v in map.walkable_neighbors(u) {
                if visited.insert(v) {
                    pq.push(Step {
                        steps: steps - 1,
                        pos: v,
                    });
                }
            }
        }

        None
    }

    fn part1(&self) -> Option<i32> {
        self.mininium_steps(self.0.start)
    }

    fn part2(&self) -> Option<i32> {
        // TODO: Perhaps there's a more time-efficient approach?
        self.0
            .lowest
            .iter()
            .flat_map(|u| self.mininium_steps(*u))
            .min()
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let task = Task::parse(&input)?;
    println!("part 1: {}", task.part1().unwrap_or_default());
    println!("part 2: {}", task.part2().unwrap_or_default());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task() -> Task {
        let input = "\
        Sabqponm
        abcryxxl
        accszExk
        acctuvwj
        abdefghi";
        Task::parse(input).unwrap()
    }

    #[test]
    fn parsing() {
        let Task(map) = task();
        assert_eq!(map.start, Position(0, 0));
        assert_eq!(map.end, Position(2, 5));
    }

    #[test]
    fn part1() {
        let task = task();
        assert_eq!(task.part1().unwrap(), 31);
    }

    #[test]
    fn part2() {
        let task = task();
        assert_eq!(task.part2().unwrap(), 29);
    }
}
