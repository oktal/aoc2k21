use std::vec::Vec;
use std::string::String;
use std::fs;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use std::result::Result;
use std::error::Error;

use std::iter::Iterator;

#[derive(Debug)]
pub(super) enum SolverError {
    Invalid,

    InputFile(PathBuf, std::io::Error),

    Generic(Box<dyn Error>),

    Test {
        got: String,
        expected: String
    },
}

type SolverResult = Result<String, SolverError>;

pub(super) trait Solver {
    fn solve(&self, lines: Vec<String>) -> SolverResult;

    fn test_expected(&self) -> &'static str;
}

pub(super) struct Day0 {
}

impl Day0 {
    pub(super) fn new() -> Box<Day0> {
        Box::new(Day0{})
    }
}

impl Solver for Day0 {
    fn solve(&self, lines: Vec<String>) -> SolverResult {
        let lines = lines
            .iter()
            .map(
                |s|
                    s.parse::<i64>()
                    .map_err(
                        |e|
                            SolverError::Generic(e.into())
                    )
            )
            .collect::<Result<Vec<_>, _>>()?;

        Ok(lines.iter().sum::<i64>().to_string())
    }

    fn test_expected(&self) -> &'static str {
        "5"
    }
}

struct PreparedSolver<'a>(Vec<String>, &'a Box<dyn Solver>);

fn prepare_solver<
        P: AsRef<Path>,
        Fn: FnOnce(PreparedSolver) -> SolverResult
    >
    (path: P, day: usize, f: Fn) -> SolverResult {

    let days:  &[Box<dyn Solver>] = &[
        Day0::new()
    ];

    let file = fs::File::open(path.as_ref()).map_err(
        |e| SolverError::InputFile(
                PathBuf::from(path.as_ref()), e
        )
    )?;

    let reader = BufReader::new(file);
    let lines = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(
            |e|
                SolverError::InputFile(
                    PathBuf::from(path.as_ref()),
                    e
                )
        )?;

    days
        .get(day)
        .ok_or(SolverError::Invalid)
        .and_then(|s| f(PreparedSolver(lines, s)))
}


pub(super) fn solve<P: AsRef<Path>>(path: P, day: usize) -> SolverResult {
    prepare_solver(path, day, |s| s.1.solve(s.0))
}

fn run_test<'a>(solver: PreparedSolver<'a>) -> SolverResult {
    let expected = solver.1.test_expected();
    let result = solver.1.solve(solver.0)?;

    if result == expected {
        Ok(result)
    } else {
        Err(SolverError::Test {
            got: result,
            expected: expected.to_string()
        })
    }
}

pub(super) fn test<P: AsRef<Path>>(path: P, day: usize) -> SolverResult {
    prepare_solver(path, day, run_test)
}

