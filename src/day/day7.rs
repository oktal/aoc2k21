use super::{Solver, SolverError, SolverResult};

struct Day7;

impl Solver for Day7 {
    fn name(&self) -> &'static str {
        "The Treachery of Whales"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        let mut positions = lines[0]
            .split(',')
            .into_iter()
            .map(|x| x.parse::<u64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SolverError::Generic(e.into()))?;

        positions.sort();
        let median_idx = (positions.len() + 1) / 2;
        let median = positions[median_idx] as i64;

        let spent_fuel: u64 = positions
            .iter()
            .map(|x| (*x as i64 - median).abs() as u64)
            .sum();

        Ok(spent_fuel.to_string())
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        let positions = lines[0]
            .split(',')
            .into_iter()
            .map(|x| x.parse::<u64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SolverError::Generic(e.into()))?;

        let max_pos = *positions.iter().max().unwrap();
        let mut spent_fuels = Vec::new();

        // Let's compute the fuel we need to spend for each possible position and move
        for pos in &positions {
            let mut spent_fuel = Vec::new();

            for i in 0..max_pos + 1 {
                let mut fuel = 0u64;
                let (src, target) = if i > *pos { (*pos, i) } else { (i, *pos) };

                (src..target).into_iter().enumerate().for_each(|(idx, _)| {
                    fuel += idx as u64 + 1;
                });

                spent_fuel.push(fuel);
            }

            spent_fuels.push(spent_fuel);
        }

        // We now compute the total fuel we need to spend for each possible move
        let mut fuels = Vec::new();

        for i in 0..max_pos + 1 {
            let mut spent = 0u64;
            for f in &spent_fuels {
                spent += f.get(i as usize).unwrap();
            }

            fuels.push(spent);
        }

        // And our answer is the minimum
        let spent_fuel = fuels.iter().min().unwrap();
        Ok(spent_fuel.to_string())
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "37",
            2 => "168",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day7)
}
