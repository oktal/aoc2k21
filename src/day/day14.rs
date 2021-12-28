use super::{Solver, SolverError, SolverResult};

use std::collections::HashMap;
use std::fmt;
use std::result::Result;

#[derive(Debug)]
enum PairInsertionError {
    MissingPair,

    MissingInsertion,
}

impl std::error::Error for PairInsertionError {}

impl fmt::Display for PairInsertionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn parse_insertion_pair(s: &str) -> Result<(String, String), PairInsertionError> {
    let mut split = s.split(" -> ");

    let pair = split.next().ok_or(PairInsertionError::MissingPair)?;
    let insertion = split.next().ok_or(PairInsertionError::MissingInsertion)?;

    Ok((pair.to_string(), insertion.to_string()))
}

fn solve(lines: Vec<String>, steps: usize) -> SolverResult {
    let template = lines.get(0).ok_or(SolverError::Generic(
        "Failed to retrieve the polymer template".into(),
    ))?;

    let insertions = lines.get(2..).ok_or(SolverError::Generic(
        "Failed to retrieve pair insertion rules".into(),
    ))?;

    let insertion_pairs = insertions
        .iter()
        .map(|l| parse_insertion_pair(&l))
        .collect::<Result<HashMap<_, _>, _>>()
        .map_err(|e| SolverError::Generic(e.into()))?;

    let mut pairs_table = HashMap::new();
    let mut index = 0usize;
    while let Some(pair) = template.as_str().get(index..index + 2) {
        if let Some(count) = pairs_table.get_mut(pair) {
            *count += 1;
        } else {
            pairs_table.insert(pair.to_string(), 1);
        }

        index += 1;
    }

    for _ in 0..steps {
        let mut new_pairs_table = HashMap::new();
        for (pair, count) in pairs_table {
            if let Some(insertion) = insertion_pairs.get(&pair) {
                let pair_bytes = pair.as_bytes();

                let insertion_bytes = insertion.as_bytes();
                let insertion = insertion_bytes[0];

                let pair_left = String::from_utf8_lossy(&[pair_bytes[0], insertion]).to_string();
                let pair_right = String::from_utf8_lossy(&[insertion, pair_bytes[1]]).to_string();

                *new_pairs_table.entry(pair_left).or_insert(0) += count;
                *new_pairs_table.entry(pair_right).or_insert(0) += count;
            } else {
                *new_pairs_table.entry(pair).or_insert(0) += count;
            }
        }

        pairs_table = new_pairs_table;
    }

    let mut occurences = HashMap::new();
    for (pair, count) in pairs_table {
        let first_char = pair.chars().next().unwrap();
        *occurences.entry(first_char).or_insert(0usize) += count;
    }

    let mut count = occurences.into_iter().collect::<Vec<_>>();
    count.sort_by(|a, b| a.1.cmp(&b.1));

    let least_common = count.first().expect("Should have at least one element");
    let most_common = count.last().expect("Should have at least one element");

    Ok(((most_common.1 - least_common.1) + 1).to_string())
}

struct Day14;

impl Solver for Day14 {
    fn name(&self) -> &'static str {
        "Extended Polymerization"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        solve(lines, 10)
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        solve(lines, 40)
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "1588",
            2 => "2188189693529",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day14)
}
