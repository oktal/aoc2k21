use super::{Solver, SolverError, SolverResult};
use std::fmt::{self, Write};

use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum FoldInstruction {
    // fold x=value
    X(usize),

    // fold y=value
    Y(usize),
}

impl FromStr for FoldInstruction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const PREFIX: &'static str = "fold along ";

        if !s.starts_with(PREFIX) {
            return Err("Invalid fold instruction");
        }

        let instruction = &s[PREFIX.len()..];
        let mut split = instruction.split('=');

        let axe = split
            .next()
            .ok_or("Missing instruction for fold instruction")?;
        let value = split.next().ok_or("Missing value for fold instruction")?;

        let axe = axe.to_lowercase();
        if !matches!(axe.as_str(), "x" | "y") {
            return Err("Invalid axe for fold instruction");
        }

        let value = value
            .parse::<usize>()
            .map_err(|_| "Invalid value for fold instruction")?;

        match axe.as_str() {
            "x" => Ok(FoldInstruction::X(value)),
            "y" => Ok(FoldInstruction::Y(value)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct Index(usize, usize);

#[derive(Debug)]
enum Instruction {
    Keep(Index),

    Fold { from: Index, to: Index },
}

impl FoldInstruction {
    fn get_instructions(&self, width: usize, height: usize) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let instruction = match self {
                    FoldInstruction::Y(fold_y) => {
                        if y < *fold_y {
                            Instruction::Keep(Index(x, y))
                        } else {
                            Instruction::Fold {
                                from: Index(x, y),
                                to: Index(x, fold_y - (y - fold_y)),
                            }
                        }
                    }
                    FoldInstruction::X(fold_x) => {
                        if x < *fold_x {
                            Instruction::Keep(Index(x, y))
                        } else {
                            Instruction::Fold {
                                from: Index(x, y),
                                to: Index(fold_x - (x - fold_x), y),
                            }
                        }
                    }
                };

                instructions.push(instruction);
            }
        }

        instructions
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Point {
    Dot,
    Invisible,
}

#[derive(Debug)]
struct Grid {
    points: Vec<Point>,

    width: usize,

    height: usize,
}

impl Grid {
    fn with_capacity(width: usize, height: usize) -> Grid {
        Grid {
            points: vec![Point::Invisible; width * height],
            width,
            height,
        }
    }

    fn add(&mut self, x: usize, y: usize) -> Option<()> {
        let index = y + x * self.height;

        let point = self.points.get_mut(index)?;
        *point = Point::Dot;

        Some(())
    }

    fn apply(self, instruction: FoldInstruction) -> Grid {
        let (new_width, new_height) = match instruction {
            FoldInstruction::X(x) => (x, self.height),
            FoldInstruction::Y(y) => (self.width, y),
        };

        let mut folded_grid = Grid::with_capacity(new_width, new_height);

        for instruction in instruction.get_instructions(self.width, self.height) {
            match instruction {
                Instruction::Keep(index) => {
                    let idx = index.1 + index.0 * self.height;
                    let new_idx = index.1 + index.0 * new_height;
                    folded_grid.points[new_idx] = self.points[idx];
                }
                Instruction::Fold { from, to } => {
                    let idx_from = from.1 + from.0 * self.height;

                    if let Point::Dot = self.points[idx_from] {
                        folded_grid.add(to.0, to.1);
                    }
                }
            }
        }

        folded_grid
    }

    fn parse(lines: Vec<String>) -> Result<(Grid, Vec<FoldInstruction>), SolverError> {
        enum ParsingState {
            Coord,
            FoldInstruction,
        }

        let mut state = ParsingState::Coord;

        let mut coords = Vec::new();
        let mut instructions = Vec::new();

        let mut max_x = 0u64;
        let mut max_y = 0u64;

        for line in lines {
            if line.is_empty() {
                state = ParsingState::FoldInstruction;
                continue;
            }

            match state {
                ParsingState::Coord => {
                    let mut split = line.split(",");

                    let x = split
                        .next()
                        .ok_or(SolverError::Generic("Missing x coordinate".into()))?;
                    let y = split
                        .next()
                        .ok_or(SolverError::Generic("Missing y coordinate".into()))?;

                    let x = x
                        .parse::<u64>()
                        .map_err(|e| SolverError::Generic(e.into()))?;
                    let y = y
                        .parse::<u64>()
                        .map_err(|e| SolverError::Generic(e.into()))?;

                    if x > max_x {
                        max_x = x;
                    }

                    if y > max_y {
                        max_y = y;
                    }

                    coords.push((x, y));
                }
                ParsingState::FoldInstruction => {
                    let instruction = FoldInstruction::from_str(line.as_str())
                        .map_err(|e| SolverError::Generic(e.into()))?;
                    instructions.push(instruction)
                }
            };
        }

        let mut grid = Grid::with_capacity(max_x as usize + 1, max_y as usize + 1);

        for coord in coords {
            grid.add(coord.0 as usize, coord.1 as usize)
                .expect("Should have been able to add the point");
        }

        Ok((grid, instructions))
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let index = y + x * self.height;
                let point = self.points[index];

                match point {
                    Point::Dot => f.write_char('#')?,
                    Point::Invisible => f.write_char('.')?,
                }
            }
            print!("\n");
        }

        Ok(())
    }
}

struct Day13;

impl Solver for Day13 {
    fn name(&self) -> &'static str {
        "Transparent Origami"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        let (grid, instructions) = Grid::parse(lines)?;
        let first_instruction = instructions
            .get(0)
            .ok_or("Empty fold instructions")
            .map_err(|e| SolverError::Generic(e.into()))?;

        let grid = grid.apply(*first_instruction);
        let visible_points = grid
            .points
            .iter()
            .filter(|p| matches!(p, Point::Dot))
            .count();
        Ok(visible_points.to_string())
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        let (mut grid, instructions) = Grid::parse(lines)?;
        for instruction in instructions {
            grid = grid.apply(instruction);
        }

        println!("{}", grid);
        Ok("".to_string())
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "17",
            2 => "",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day13)
}
