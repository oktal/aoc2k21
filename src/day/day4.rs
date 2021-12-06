use super::{Solver, SolverError, SolverResult};

struct Day4;

mod bingo {
    use std::str::FromStr;

    use std::fmt;
    use std::result::Result;

    #[derive(Debug)]
    enum Cell {
        Marked(u32),
        Unmarked(u32),
    }

    impl Cell {
        fn value(&self) -> u32 {
            match self {
                Cell::Marked(v) => *v,
                Cell::Unmarked(v) => *v,
            }
        }

        fn is_marked(&self) -> bool {
            matches!(self, Cell::Marked(_))
        }

        fn mark(&mut self) -> bool {
            match self {
                Cell::Unmarked(v) => {
                    *self = Cell::Marked(*v);
                    true
                }
                Cell::Marked(_) => false,
            }
        }
    }

    impl FromStr for Cell {
        type Err = std::num::ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse::<u32>().map(Self::Unmarked)
        }
    }

    #[derive(Debug)]
    pub(super) enum ParseBoardError {
        InvalidCell(std::num::ParseIntError),

        InvalidMatrix(usize, usize),
    }

    impl fmt::Display for ParseBoardError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl std::error::Error for ParseBoardError {}

    pub(super) trait State {}

    pub(super) struct Invalid {}

    #[derive(Debug)]
    pub(super) struct Ready {
        /// The cells of the Bingo board
        cells: Vec<Cell>,

        /// The number of rows of the board
        rows: usize,

        /// The number of columns of the board
        columns: usize,
    }

    #[derive(Debug)]
    pub(super) struct Win {
        /// The score of the winning board
        score: u32,
    }

    impl State for Invalid {}
    impl State for Ready {}
    impl State for Win {}

    pub(super) enum Drawn {
        Again(Board<Ready>),
        Won(Board<Win>),
    }

    #[derive(Debug)]
    pub(super) struct Board<S: State> {
        state: Box<S>,
    }

    struct RowIterator<'a> {
        /// The cells we want to iterate on
        cells: &'a [Cell],

        /// The row we're iterator
        row: usize,

        /// The total number of rows
        rows: usize,

        /// The current cell we are at
        current: usize,
    }

    impl<'a> Iterator for RowIterator<'a> {
        type Item = &'a Cell;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current >= self.rows {
                None
            } else {
                let cell = &self.cells[self.current * self.rows + self.row];
                self.current += 1;
                Some(cell)
            }
        }
    }

    struct ColumnIterator<'a> {
        /// The cells we want to iterate on
        cells: &'a [Cell],

        /// The columns we're iterator
        column: usize,

        /// The total number of rows
        columns: usize,

        /// The current cell we are at
        current: usize,
    }

    impl<'a> Iterator for ColumnIterator<'a> {
        type Item = &'a Cell;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current >= self.columns {
                None
            } else {
                let cell = &self.cells[self.column * self.columns + self.current];
                self.current += 1;
                Some(cell)
            }
        }
    }

    impl Board<Invalid> {
        pub(super) fn parse(lines: Vec<String>) -> Result<Board<Ready>, ParseBoardError> {
            let rows = lines.len();

            let mut cells = Vec::new();
            for row in lines {
                let columns = Board::parse_row(row)?;
                if columns.len() != rows {
                    return Err(ParseBoardError::InvalidMatrix(rows, columns.len()));
                }
                cells.extend(columns);
            }

            let state = Box::new(Ready {
                cells,
                rows,
                columns: rows,
            });

            Ok(Board::<Ready> { state })
        }

        fn parse_row(row: String) -> Result<Vec<Cell>, ParseBoardError> {
            row.split(' ')
                .filter(|l| !l.is_empty())
                .map(Cell::from_str)
                .collect::<Result<Vec<_>, _>>()
                .map_err(ParseBoardError::InvalidCell)
        }
    }

    impl Board<Ready> {
        pub(super) fn draw(mut self, n: u32) -> Drawn {
            let cell = self.state.cells.iter_mut().find(|c| c.value() == n);
            if let Some(cell) = cell {
                cell.mark();
            }

            let rows = self.state.rows;
            let columns = self.state.columns;

            // Let's check if we won
            let mut won = true;

            // First, check the rows
            for r in 0..rows {
                won = self.iter_row(r).all(|c| c.is_marked());
                if won {
                    break;
                }
            }

            // We didn't have a winner row, check the columns
            if !won {
                for c in 0..columns {
                    won = self.iter_column(c).all(|c| c.is_marked());
                    if won {
                        break;
                    }
                }
            }

            // We won, let's compute our score
            if won {
                let score: u32 = self
                    .state
                    .cells
                    .iter()
                    .filter(|c| !c.is_marked())
                    .map(|c| c.value())
                    .sum();
                Drawn::Won(Board::<Win> {
                    state: Box::new(Win { score }),
                })
            } else {
                Drawn::Again(self)
            }
        }

        fn iter_row<'a>(&'a self, row: usize) -> RowIterator<'a> {
            RowIterator {
                cells: &self.state.cells.as_slice(),
                row: row,
                rows: self.state.rows,
                current: 0
            }
        }

        fn iter_column<'a>(&'a self, column: usize) -> ColumnIterator<'a> {
            ColumnIterator {
                cells: &self.state.cells.as_slice(),
                column: column,
                columns: self.state.columns,
                current: 0
            }
        }
    }

    impl Board<Win> {
        pub(super) fn score(&self) -> u32 {
            self.state.score
        }
    }
}

// Play and return the scores of winning boards by order
fn play(lines: Vec<String>) -> Result<Vec<u32>, SolverError>  {
    let game = lines[0]
        .split(',')
        .map(|x| x.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SolverError::Generic(e.into()))?;

    let mut boards = Vec::new();
    let splits = lines[1..].split(|l| l.is_empty());

    for split in splits {
        if split.is_empty() {
            continue;
        }
        let board = bingo::Board::parse(Vec::from(split))
            .map_err(|e| SolverError::Generic(e.into()))?;
        boards.push(board)
    }

    let mut scores = Vec::new();

    for g in game {
        let mut new_boards = Vec::new();

        for board in boards.into_iter() {
            match board.draw(g) {
                bingo::Drawn::Again(b) => new_boards.push(b),
                bingo::Drawn::Won(b) => scores.push(b.score() * g)
            };
        }

        boards = new_boards;
    }

    Ok(scores)
}

impl Solver for Day4 {
    fn name(&self) -> &'static str {
        "Giant Squid"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        let scores = play(lines)?;
        scores
            .get(0)
            .ok_or(SolverError::Generic("Could not determine a winner board".into()))
            .map(|s| s.to_string())
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        let scores = play(lines)?;
        let len = scores.len();
        let last = if len > 0 { len - 1 } else { 0 };
        scores
            .get(last)
            .ok_or(SolverError::Generic("Could not determine a winner board".into()))
            .map(|s| s.to_string())
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "4512",
            2 => "1924",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day4)
}
