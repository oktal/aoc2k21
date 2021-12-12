use super::{Solver, SolverError, SolverResult};
use std::convert::TryFrom;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

struct Day8;

#[derive(Debug)]
enum Error {
    MissingInput,
    MissingOutput,
    InvalidSegment(char),
    InvalidWiring(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Segment {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl TryFrom<char> for Segment {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'a' => Ok(Self::A),
            'b' => Ok(Self::B),
            'c' => Ok(Self::C),
            'd' => Ok(Self::D),
            'e' => Ok(Self::E),
            'f' => Ok(Self::F),
            'g' => Ok(Self::G),
            _ => Err(Error::InvalidSegment(c))
        }
    }
}

#[derive(Debug, Clone)]
struct Wiring {
    segments: Vec<Segment>
}

impl FromStr for Wiring {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segments =
            s.chars().map(Segment::try_from).collect::<Result<Vec<_>, _>>()?;
        Ok(Wiring { segments })
    }
}

#[derive(Debug, Clone)]
struct Digit {
    wiring: Wiring,
    value: Option<u32>
}

impl Digit {
    fn is_unique(&self) -> bool {
        if let Some(val) = self.value {
            matches!(val, 1 | 4 | 7 | 8)
        } else {
            false
        }
    }

    fn common_segments(&self, other: &Digit) -> Vec<Segment> {
        let mut common = Vec::new();
        for segment in &self.wiring.segments {
            if let Some(_) = &other.wiring.segments.iter().find(|&&s| *segment == s) {
                common.push(*segment);
            }
        }

        common 
    }
}

impl FromStr for Digit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segments =
            s.chars().map(Segment::try_from).collect::<Result<Vec<_>, _>>()?;

        let value = match segments.len() {
            1 => Err(Error::InvalidWiring(s.to_owned())),
            2 => Ok(Some(1)), // Only 1 has 2 segments
            3 => Ok(Some(7)), // Only 7 has 3 segments 
            4 => Ok(Some(4)), // Only 4 has 4 segments
            5 => Ok(None), // Can be 2, 3, 5
            6 => Ok(None), // Can be 0, 6, 9 
            7 => Ok(Some(8)), // Only 8 has 7 segments
            _ => Err(Error::InvalidWiring(s.to_owned())),
        }?;


        Ok(Digit {
            wiring: Wiring { segments },
            value
        })
    }
}

#[derive(Debug)]
struct Entry {
    pattern: Vec<Digit>,
    output: Vec<Digit>
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s
            .split('|')
            .into_iter();

        let pattern = split.next().ok_or(Error::MissingInput)?;
        let output = split.next().ok_or(Error::MissingOutput)?;

        let pattern =
            pattern.split(' ').filter(|x| !x.is_empty()).map(Digit::from_str).collect::<Result<Vec<_>, _>>()?;

        let output =
            output.split(' ').filter(|x| !x.is_empty()).map(Digit::from_str).collect::<Result<Vec<_>, _>>()?;

        Ok(Entry { pattern, output })
    }
}


fn solve_entry(entry: &Entry) -> u64 {
    let mut known_digits = {
        let known_digits =
            entry
            .pattern
            .iter()
            .filter(|p| p.value.is_some());

        let mut groups = HashMap::new();
        for d in known_digits {
            groups.entry(d.value.unwrap()).or_insert(d.clone());
        }

        groups
    };

    let mut unsolved = {
        let unsolved =
            entry
            .pattern
            .iter()
            .filter(|p| p.value.is_none());

        let mut groups = HashMap::new();
        for d in unsolved {
            let len = d.wiring.segments.len();
            if len == 5 {
                for v in &[2, 3, 5] {
                    groups.entry(*v as u32).or_insert(Vec::new()).push(d.clone());
                }
            }
            else if len == 6 {
                for v in &[0, 6, 9] {
                    groups.entry(*v as u32).or_insert(Vec::new()).push(d.clone());
                }
            }
        }

        groups
    };

    let mut solved = Vec::new();
    loop {
        if known_digits.len() == 10 {
            break;
        }

        for (digit, patterns) in &mut unsolved {

            if patterns.len() == 1 {
                let mut pattern = patterns[0].clone();
                pattern.value = Some(*digit);
                solved.push(pattern.clone());
                known_digits.insert(*digit, pattern.clone());
                continue;
            }

            let ps = patterns.iter().filter(|&p1| solved.iter().find(|&p2| p1.wiring.segments == p2.wiring.segments).is_none());


            let mut possible = Vec::new();
            for pattern in ps {
                let mut common_segments: [Option<usize>; 3] = [None; 3];
                if let Some(one) = known_digits.get(&1) {
                    let common = pattern.common_segments(one);
                    common_segments[0] = Some(common.len());
                }
                if let Some(four) = known_digits.get(&4) {
                    let common = pattern.common_segments(four);
                    common_segments[1] = Some(common.len());
                }

                if let Some(seven) = known_digits.get(&7) {
                    let common = pattern.common_segments(seven);
                    common_segments[2] = Some(common.len());
                }

                if *digit == 0 {
                    if let &[Some(one), Some(four), Some(seven)] = &common_segments {
                        if one >= 2 && four >= 3 && seven >= 3 {
                            let pattern = pattern.clone();
                            possible.push(pattern.clone());
                        }
                    }
                }
                else if *digit == 2 {
                    if let &[Some(one), Some(four), Some(seven)] = &common_segments {
                        if one >= 1 && four >= 2 && seven >= 2 {
                            possible.push(pattern.clone());
                        }
                    }
                }
                else if *digit == 3 {
                    if let &[Some(one), Some(four), Some(seven)] = &common_segments {
                        if one >= 2 && four >= 3 && seven >= 2 {
                            possible.push(pattern.clone());
                        }
                    }
                }
                else if *digit == 5 {
                    if let &[Some(one), Some(four), Some(seven)] = &common_segments {
                        if one >= 1 && four >= 3 && seven >= 2 {
                            possible.push(pattern.clone());
                        }
                    }
                }
                else if *digit == 6 {
                    if let &[Some(one), Some(four), Some(seven)] = &common_segments {
                        if one >= 1 && four >= 3 && seven >= 2 {
                            possible.push(pattern.clone());
                        }
                    }
                }
                else if *digit == 9 {
                    if let &[Some(one), Some(four), Some(seven)] = &common_segments {
                        if one >= 2 && four >= 4 && seven >= 3 {
                            possible.push(pattern.clone());
                        }
                    }
                }
            }

            *patterns = possible;
        }
    }


    let digits = known_digits.values().collect::<Vec<_>>();
    let mut result = 0u64;
    let mut mul = 1;
    for digit in entry.output.iter() {
        let value = match digit.value {
            Some(value) => value,
            None => {
                let len = digit.wiring.segments.len();
                let d = digits
                    .iter()
                    .find(|&&d| {
                        let common = d.common_segments(digit);
                        len == common.len() && len == d.wiring.segments.len()
                    });

                d.unwrap().value.unwrap()
            }
        };

        result += (value as u64) * (1000 / mul);
        mul *= 10;
    }

    result
}


impl Solver for Day8 {
    fn name(&self) -> &'static str {
        "Seven Segment Search"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        let entries = lines
            .into_iter()
            .map(|l| Entry::from_str(&l))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SolverError::Generic(e.into()))?;

        let unique_output_digits: usize =
            entries
            .iter()
            .map(|e| e.output.iter().filter(|d| d.is_unique()).count())
            .sum();

        Ok(unique_output_digits.to_string())
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        let entries = lines
            .into_iter()
            .map(|l| Entry::from_str(&l))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SolverError::Generic(e.into()))?;

        let sum: u64 = entries.iter().map(solve_entry).sum();
        Ok(sum.to_string())
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "26",
            2 => "61229",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day8)
}
