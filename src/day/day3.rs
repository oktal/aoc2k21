use super::{Solver, SolverError, SolverResult};

use std::fmt;
use std::result::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Error {
    /// We've hit the recursion limit when attempting to retrieve the oxygen generator and CO2
    /// scrubber ratings
    RecursionLimit(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

struct Day3;

type ReportType = u32;

fn parse_reports(lines: Vec<String>) -> Result<Vec<ReportType>, SolverError> {
    lines
        .iter()
        .map(|l| ReportType::from_str_radix(l.as_str(), 2))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SolverError::Generic(e.into()))
}

fn get_size(reports: &[ReportType]) -> Option<u32> {
    // Find the total number of bits that we need to compute based on the maximum
    // line size we got
    reports
        .iter()
        .map(|x| ReportType::BITS - x.leading_zeros())
        .max()
}

const MAX_REC: u32 = 1_00;

fn rating_rec<F: Fn(usize, usize) -> bool>(
    reports: &[ReportType],
    bit: u32,
    size: u32,
    f: F,
) -> Option<ReportType> {
    if reports.is_empty() || bit >= MAX_REC {
        return None;
    }

    if reports.len() == 1 {
        return Some(reports[0]);
    }

    let mask = (1 as ReportType) << (size - bit - 1);

    let mut zeros = Vec::new();
    let mut ones = Vec::new();

    for report in reports {
        if report & mask == mask {
            ones.push(*report);
        } else {
            zeros.push(*report);
        }
    }

    if f(ones.len(), zeros.len()) {
        rating_rec(ones.as_slice(), bit + 1, size, f)
    } else {
        rating_rec(zeros.as_slice(), bit + 1, size, f)
    }
}

impl Solver for Day3 {
    fn name(&self) -> &'static str {
        "Binary Diagnostic"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        let reports = parse_reports(lines)?;
        let size = get_size(reports.as_slice()).unwrap();

        let mut gamma_rate = ReportType::default();
        let mut epsilon_rate = ReportType::default();

        for bit in 0..size {
            let mask = (1 as ReportType) << bit;

            let mut zero_count = 0;
            let mut one_count = 0;

            for report in &reports {
                if report & mask == mask {
                    one_count += 1
                } else {
                    zero_count += 1
                }
            }

            if one_count > zero_count {
                gamma_rate |= mask;
            } else {
                epsilon_rate |= mask;
            }
        }

        Ok((gamma_rate * epsilon_rate).to_string())
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        let reports = parse_reports(lines)?;
        let size = get_size(reports.as_slice()).unwrap();

        let oxygen_generator = rating_rec(reports.as_slice(), 0, size, |ones, zeros| ones >= zeros)
            .ok_or(SolverError::Generic(Error::RecursionLimit(MAX_REC).into()))?;
        let co2_scrubber = rating_rec(reports.as_slice(), 0, size, |ones, zeros| zeros > ones)
            .ok_or(SolverError::Generic(Error::RecursionLimit(MAX_REC).into()))?;

        Ok((oxygen_generator * co2_scrubber).to_string())
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "198",
            2 => "230",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day3)
}
