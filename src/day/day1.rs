use super::{Solver, SolverError, SolverResult};

use std::result::Result;

struct Day1;

impl Day1 {
    pub(super) fn new() -> Box<Day1> {
        Box::new(Day1{})
    }
}

fn solve(depths: impl Iterator<Item = u64>) -> SolverResult {
    let mut increase_count = 0usize;
    let mut previous: Option<u64> = None;

    for depth in depths {
        if let Some(previous) = previous {
            if depth > previous {
                increase_count += 1;
            }
        }

        previous = Some(depth);
    }

    Ok(increase_count.to_string())
}

fn parse_depths(lines: Vec<String>) -> Result<Vec<u64>, SolverError> {
    lines
        .iter()
        .map(|l| l.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SolverError::Generic(e.into()))
}

impl Solver for Day1 {
    fn name(&self) -> &'static str {
        "Sonar Sweep"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        parse_depths(lines)
            .and_then(|d| solve(d.into_iter()))
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        let depths = parse_depths(lines)?;
        let window_sums = 
            depths
                .as_slice()
                .windows(3)
                .map(|w| w.iter().sum());
        solve(window_sums)
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "7",
            2 => "5",
            _ => unreachable!()
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Day1::new()
}
