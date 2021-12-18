use super::{Solver, SolverError, SolverResult};

struct Day11;

#[derive(Debug, Copy, Clone)]
enum OctopusState {
    Flashed(usize),
    Ready(u32),
}

#[derive(Debug)]
struct Octopus(OctopusState);

impl Octopus {
    fn increase(&mut self) -> OctopusState {
        self.0 = match self.0 {
            OctopusState::Ready(x) => {
                if x == 9 {
                    OctopusState::Flashed(1)
                } else {
                    OctopusState::Ready(x + 1)
                }
            }
            OctopusState::Flashed(x) => OctopusState::Flashed(x + 1),
        };

        self.0
    }

    fn flashed(&self) -> bool {
        matches!(self.0, OctopusState::Flashed(_))
    }

    fn reset(&mut self) {
        self.0 = match self.0 {
            OctopusState::Flashed(_) => OctopusState::Ready(0),
            x => x,
        };
    }
}

struct Grid {
    octopuses: Vec<Octopus>,

    rows: usize,

    columns: usize,
}

impl Grid {
    fn get_octopus_at_mut(&mut self, x: usize, y: usize) -> Option<&mut Octopus> {
        self.octopuses.get_mut(x * self.columns + y)
    }

    fn reset(&mut self) -> usize {
        let mut total_flashed = 0;

        for octopus in &mut self.octopuses {
            if octopus.flashed() {
                octopus.reset();
                total_flashed += 1;
            }
        }

        total_flashed
    }

    fn len(&self) -> usize {
        self.octopuses.len()
    }

    fn get_adjacent(&self, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
        const DIRECTIONS: &'static [(i32, i32)] = &[
            (0, -1),
            (0, 1),
            (-1, 0),
            (1, 0),
            (1, -1),
            (1, 1),
            (-1, -1),
            (-1, 1),
        ];

        let rows = self.rows - 1;
        let columns = self.columns - 1;

        DIRECTIONS.iter().filter_map(move |d| {
            let (d_x, d_y) = d;

            let (x, y) = {
                let x = if *d_x < 0 {
                    x.checked_sub(d_x.abs() as usize)
                } else {
                    Some(x + *d_x as usize)
                };

                let y = if *d_y < 0 {
                    y.checked_sub(d_y.abs() as usize)
                } else {
                    Some(y + *d_y as usize)
                };

                (x, y)
            };

            match (x, y) {
                (Some(x), Some(y)) => {
                    if x > rows || y > columns {
                        None
                    } else {
                        Some((x, y))
                    }
                }
                _ => None,
            }
        })
    }
}

fn parse_line(line: &str) -> Option<Vec<Octopus>> {
    line.chars()
        .map(|c| c.to_digit(10).map(|d| Octopus(OctopusState::Ready(d))))
        .collect::<Option<Vec<_>>>()
}

fn parse_grid(lines: Vec<String>) -> Result<Grid, SolverError> {
    let mut octopuses = Vec::new();
    let mut columns = 0usize;
    for line in &lines {
        octopuses.extend(parse_line(&line).ok_or(SolverError::Generic("Invalid line".into()))?);

        if columns > 0 && line.len() != columns {
            return Err(SolverError::Generic("Not a grid".into()));
        }

        columns = line.len();
    }

    Ok(Grid {
        octopuses,
        rows: lines.len(),
        columns,
    })
}

fn increase(grid: &mut Grid, x: usize, y: usize) {
    let octopus = grid.get_octopus_at_mut(x, y).unwrap();
    let state = octopus.increase();

    // This is the first time this little guy flashes, increase adjacent
    if let OctopusState::Flashed(1) = state {
        for (adj_x, adj_y) in grid.get_adjacent(x, y) {
            increase(grid, adj_x, adj_y);
        }
    }
}

fn run_step(grid: &mut Grid) -> usize {
    for i in 0..grid.rows {
        for j in 0..grid.columns {
            increase(grid, i, j);
        }
    }

    grid.reset()
}

impl Solver for Day11 {
    fn name(&self) -> &'static str {
        "Dumbo Octopus"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        let mut grid = parse_grid(lines)?;
        let mut total_flashes = 0usize;
        for _step in 0..100 {
            total_flashes += run_step(&mut grid);
        }

        Ok(total_flashes.to_string())
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        let mut grid = parse_grid(lines)?;
        let step = {
            let mut step = 1usize;

            loop {
                let flashes = run_step(&mut grid);
                // Did they all flash ?
                if flashes == grid.len() {
                    break;
                }

                step += 1;
            }

            step
        };

        Ok(step.to_string())
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "1656",
            2 => "195",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day11)
}
