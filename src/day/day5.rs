use super::{Solver, SolverError, SolverResult};

use regex::Regex;
use std::fmt;
use std::fmt::Write;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Point {
    x: u64,

    y: u64,
}

#[derive(Debug)]
struct Line {
    start: Point,

    end: Point,
}

#[derive(Debug)]
struct Diagram {
    points: Vec<usize>,

    rows: usize,

    columns: usize,
}

impl fmt::Display for Diagram {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.rows {
            for j in 0..self.columns {
                let value = self.value(j, i);
                if value == 0 {
                    formatter.write_char('.')?;
                } else {
                    write!(formatter, "{}", value)?;
                }
            }
            formatter.write_char('\n')?;
        }

        Ok(())
    }
}

impl Diagram {
    fn new(rows: usize, columns: usize) -> Diagram {
        Diagram {
            points: vec![0usize; rows * columns],
            rows,
            columns,
        }
    }

    fn apply(&mut self, line: &Line, diag: bool) {
        let x1 = line.start.x as usize;
        let y1 = line.start.y as usize;

        let x2 = line.end.x as usize;
        let y2 = line.end.y as usize;

        if x1 == x2 {
            let ys = (y1 as i64 - y2 as i64).abs() as usize;

            let y1 = if y1 > y2 { y2 } else { y1 };

            for y in 0..ys + 1 {
                self.incr(x1, y1 + y);
            }
        } else if y1 == y2 {
            let xs = (x1 as i64 - x2 as i64).abs() as usize;

            let x1 = if x1 > x2 { x2 } else { x1 };

            for x in 0..xs + 1 {
                self.incr(x1 + x, y1);
            }
        } else if diag {
            let mut cur = line.start;
            let end = line.end;

            let mut x_i = 0;
            let mut y_i = 0;

            while cur != end {
                let new_x = if x1 > x2 {
                    x1 - x_i
                } else {
                    x1 + x_i
                };

                let new_y = if y1 > y2 {
                    y1 - y_i
                } else {
                    y1 + y_i
                };

                x_i += 1;
                y_i += 1;

                cur = Point{x: new_x as u64, y: new_y as u64};
                self.incr(new_x, new_y);
            }
        }
    }

    fn value(&self, x: usize, y: usize) -> usize {
        self.points[self.index(x, y)]
    }

    fn incr(&mut self, x: usize, y: usize) {
        let index = self.index(x, y);
        self.points[index] += 1;
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.columns + x
    }
}

struct Day5 {
    re: Regex,
}

impl Day5 {
    fn parse_lines(&self, lines: Vec<String>) -> Result<Vec<Line>, SolverError> {
        lines
            .iter()
            .map(|s| self.parse_line(s))
            .collect::<Option<Vec<_>>>()
            .ok_or(SolverError::Generic("Failed to parse lines".into()))
    }

    fn parse_line(&self, s: &str) -> Option<Line> {
        let captures = self.re.captures(s)?;

        match (
            captures.name("x1"),
            captures.name("y1"),
            captures.name("x2"),
            captures.name("y2"),
        ) {
            (Some(x1), Some(y1), Some(x2), Some(y2)) => {
                let x1 = x1.as_str();
                let y1 = y1.as_str();

                let x2 = x2.as_str();
                let y2 = y2.as_str();

                let x1 = x1.parse::<u64>().unwrap();
                let y1 = y1.parse::<u64>().unwrap();

                let x2 = x2.parse::<u64>().unwrap();
                let y2 = y2.parse::<u64>().unwrap();

                let start = Point { x: x1, y: y1 };

                let end = Point { x: x2, y: y2 };

                Some(Line { start, end })
            }
            _ => None,
        }
    }
}

fn solve(lines: Vec<Line>, diag: bool) -> SolverResult {
    let mut max_x = 0;
    let mut max_y = 0;

    for line in &lines {
        if line.start.x > max_x {
            max_x = line.start.x
        }

        if line.end.x > max_x {
            max_x = line.end.x
        }

        if line.start.y > max_y {
            max_y = line.start.y
        }

        if line.end.y > max_y {
            max_y = line.end.y
        }
    }

    let mut diagram = Diagram::new(max_x as usize + 1, max_y as usize + 1);
    lines.iter().for_each(|l| diagram.apply(l, diag));

    let count = diagram.points.iter().filter(|&x| *x >= 2).count();

    Ok(count.to_string())
}

impl Solver for Day5 {
    fn name(&self) -> &'static str {
        "Hydrothermal Venture"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        solve(self.parse_lines(lines)?, false)
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        solve(self.parse_lines(lines)?, true)
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "5",
            2 => "12",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    let re = Regex::new(r"(?P<x1>\d+),(?P<y1>\d+).*?->.*?(?P<x2>\d+),(?P<y2>\d+)").unwrap();
    Box::new(Day5 { re })
}
