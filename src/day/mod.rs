use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::string::String;
use std::vec::Vec;

use std::error::Error;
use std::result::Result;

use std::iter::Iterator;

mod day1;
mod day2;
mod day3;

#[derive(Debug)]
pub(super) enum SolverError {
    UnknownDay(usize),
    InvalidPart(usize),

    InputFile(PathBuf, std::io::Error),

    Generic(Box<dyn Error>),

    Test { got: String, expected: String },
}

type SolverResult = Result<String, SolverError>;

pub(super) trait Solver {
    fn name(&self) -> &'static str;

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult;

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult;

    fn test_expected(&self, part: usize) -> &'static str;
}

struct PreparedSolver<'a>(Vec<String>, &'a Box<dyn Solver>);

pub(super) fn name(day: usize) -> Option<&'static str> {
    let days: &[Box<dyn Solver>] = &[day1::new(), day2::new(), day3::new()];

    days.get(day - 1).map(|d| d.name())
}

fn prepare_solver<P: AsRef<Path>, Fn: FnOnce(PreparedSolver) -> SolverResult>(
    path: P,
    day: usize,
    f: Fn,
) -> SolverResult {
    let days: &[Box<dyn Solver>] = &[day1::new(), day2::new(), day3::new()];

    let file = fs::File::open(path.as_ref())
        .map_err(|e| SolverError::InputFile(PathBuf::from(path.as_ref()), e))?;

    let reader = BufReader::new(file);
    let lines = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SolverError::InputFile(PathBuf::from(path.as_ref()), e))?;

    days.get(day - 1)
        .ok_or(SolverError::UnknownDay(day))
        .and_then(|s| f(PreparedSolver(lines, s)))
}

fn run_solver<'a>(solver: PreparedSolver<'a>, part: usize) -> SolverResult {
    match part {
        1 => solver.1.solve_part1(solver.0),
        2 => solver.1.solve_part2(solver.0),
        _ => Err(SolverError::InvalidPart(part)),
    }
}

pub(super) fn solve<P: AsRef<Path>>(path: P, day: usize, part: usize) -> SolverResult {
    prepare_solver(path, day, |s| run_solver(s, part))
}

fn run_test<'a>(solver: PreparedSolver<'a>, part: usize) -> SolverResult {
    let expected = solver.1.test_expected(part);
    let result = if part == 1 {
        solver.1.solve_part1(solver.0)
    } else {
        solver.1.solve_part2(solver.0)
    }?;

    if result == expected {
        Ok(result)
    } else {
        Err(SolverError::Test {
            got: result,
            expected: expected.to_string(),
        })
    }
}

pub(super) fn test<P: AsRef<Path>>(path: P, day: usize, part: usize) -> SolverResult {
    prepare_solver(path, day, |s| run_test(s, part))
}
