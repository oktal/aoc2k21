use super::{Solver, SolverError, SolverResult};
use std::collections::HashSet;

#[derive(Debug)]
struct Heightmap {
    positions: Vec<u32>,

    rows: usize,

    columns: usize
}

impl Heightmap {
    fn position_at(&self, x: usize, y: usize) -> u32 {
        self.positions[x * self.columns + y]
    }

    fn get_adj_index(&self, x: usize, y: usize) -> impl Iterator<Item = Option<(usize, usize)>> {
        const DIRECTIONS: &'static [(i32, i32)] = &[(0, -1), (0, 1), (-1, 0), (1, 0)];

        let rows = self.rows - 1;
        let columns = self.columns - 1;

        DIRECTIONS.iter().map(move |d| {
            let (d_x, d_y) = d;

            let (x, y) = {
                let x = if *d_x < 0 {
                    x.checked_sub(d_x.abs() as usize)
                } else {
                    Some(x + *d_x as usize)
                };

                let y = if *d_y < 0 {
                    y.checked_sub(d_y.abs() as usize)
                } else {
                    Some(y + *d_y as usize)
                };

                (x, y)
            };

            match (x, y) {
                (Some(x), Some(y)) => {
                    if x > rows || y > columns {
                        None
                    } else {
                        Some((x, y))
                    }
                },
                _ => None
            }
        })
    }
}

fn parse_line(line: &str) -> Option<Vec<u32>> {
    line
    .chars()
    .map(|c| c.to_digit(10))
    .collect::<Option<Vec<u32>>>()
}

fn parse_heightmap(lines: Vec<String>) -> Result<Heightmap, SolverError> {
    let mut positions = Vec::new();
    let mut columns = 0usize;
    for line in &lines {
        let cols = parse_line(&line).ok_or(SolverError::Generic("Invalid line".into()))?;
        columns = cols.len();

        positions.extend(cols);
    }

    Ok(Heightmap {
        positions,
        rows: lines.len(),
        columns,
    })
}

fn walk_basin_rec(map: &Heightmap, x: usize, y: usize, previous: u32, walked: &mut HashSet<(usize, usize)>) {
    let adj_indexes = map.get_adj_index(x, y); 
    for adj_index in adj_indexes {
        if let Some(index) = adj_index {
            let value = map.position_at(index.0, index.1);

            if value > previous && value < 9 {
                walked.insert(index);
                walk_basin_rec(map, index.0, index.1, value, walked);
            }
        }
    }
}

fn walk_basin(map: &Heightmap, x: usize, y: usize, current: u32) -> usize {
    let mut walked = HashSet::new();

    walk_basin_rec(map, x, y, current, &mut walked);
    walked.len() + 1
}

struct Day9;

impl Solver for Day9 {
    fn name(&self) -> &'static str {
        "Smoke Basin"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        let heightmap = parse_heightmap(lines)?;

        let mut res = 0u64;
        for i in 0..heightmap.rows {
            for j in 0..heightmap.columns {
                let current = heightmap.position_at(i, j);
                let adj_index = heightmap.get_adj_index(i, j);

                let mut adj_values = adj_index.map(|idx| {
                    idx.map(|(x, y)| heightmap.position_at(x, y))
                });

                let is_low = adj_values.all(|x| {
                    x.map(|v| current < v).unwrap_or(true)
                });

                if is_low {
                    res += (current + 1) as u64;
                }
            }
        }

        Ok(res.to_string())
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        let heightmap = parse_heightmap(lines)?;

        let mut basins = Vec::new();

        for i in 0..heightmap.rows {
            for j in 0..heightmap.columns {
                let current = heightmap.position_at(i, j);
                let adj_index = heightmap.get_adj_index(i, j);

                let mut adj_values = adj_index.map(|idx| {
                    idx.map(|(x, y)| heightmap.position_at(x, y))
                });

                let is_low = adj_values.all(|x| {
                    x.map(|v| current < v).unwrap_or(true)
                });

                if is_low {
                    let len = walk_basin(&heightmap, i, j, current);
                    basins.push(len);
                }
            }
        }


        basins.sort();
        let res = basins.iter().rev().take(3).fold(1, |acc, x| acc * x);

        Ok(res.to_string())
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "15",
            2 => "1134",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day9)
}
