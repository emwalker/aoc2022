// From https://github.com/noah-clements/AoC2022/blob/master/day18/day18.py
use crate::{Cube, Input, Int};
use color_eyre::Result;
use rustc_hash::FxHashSet;

pub struct Task {
    input: Input,
}

impl Task {
    pub fn surface_area(&self) -> Int {
        let cubes = self.input.0.clone();
        let n = cubes.len();
        let mut ans: Int = 6 * n as Int;

        for i in 0..n {
            let a = cubes[i];
            for b in cubes.iter().take(n).skip(i + 1) {
                ans -= 2 * a.adjacent(b) as Int;
            }
        }

        ans
    }

    pub fn exposed_area(&self) -> Int {
        let cubes = self.input.0.clone();

        let (min_x, max_x) = (
            cubes.iter().map(|c| c.x).min().expect("a min") - 1,
            cubes.iter().map(|c| c.x).max().expect("a max") + 2,
        );
        let (min_y, max_y) = (
            cubes.iter().map(|c| c.y).min().expect("a min") - 1,
            cubes.iter().map(|c| c.y).max().expect("a max") + 2,
        );
        let (min_z, max_z) = (
            cubes.iter().map(|c| c.z).min().expect("a min") - 1,
            cubes.iter().map(|c| c.z).max().expect("a max") + 2,
        );

        let in_bounds = |&Cube { x, y, z }: &Cube| -> bool {
            let good_x = min_x <= x && x <= max_x;
            let good_y = min_y <= y && y <= max_y;
            let good_z = min_z <= z && z <= max_z;
            good_x && good_y && good_z
        };

        let mut water = vec![Cube {
            x: min_x,
            y: min_y,
            z: min_z,
        }];
        let mut visited = FxHashSet::<Cube>::default();
        let mut ans = 0;

        let deltas = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];

        while let Some(cube) = water.pop() {
            if visited.contains(&cube) {
                continue;
            }
            visited.insert(cube);

            for &dxyz in deltas.iter() {
                let neighbor = cube.shift(dxyz);
                if in_bounds(&neighbor) {
                    if cubes.contains(&neighbor) {
                        ans += 1;
                    } else {
                        water.push(neighbor);
                    }
                }
            }
        }

        ans
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
    fn part2() {
        let task = parse(EXAMPLE).unwrap();
        assert_eq!(task.exposed_area(), 58);
    }
}
