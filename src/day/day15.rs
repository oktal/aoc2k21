use super::{Solver, SolverError, SolverResult};

struct Day15;

impl Solver for Day15 {
    fn name(&self) -> &'static str {
        todo!();
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        todo!();
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        todo!();
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "",
            2 => "",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day15)
}
