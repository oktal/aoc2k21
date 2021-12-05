use super::{Solver, SolverError, SolverResult};

use std::error::Error;
use std::fmt;

use std::result::Result;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParseCommandError {
    /// The command is empty
    Empty,

    /// Unknown command
    Unknown(String),

    /// Missing argument for the command
    MissingArgument(String),

    /// Failed to parse argument for command
    ArgumentParseError(std::num::ParseIntError),
}

impl fmt::Display for ParseCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParseCommandError {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Command {
    Forward(usize),
    Down(usize),
    Up(usize)
}

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut args = s.split(' ');
        let (command, arg) = (args.next(), args.next());

        match (command, arg) {
            (None, _) => Err(ParseCommandError::Empty),
            (Some(command), None) => Err(ParseCommandError::MissingArgument(command.to_string())),
            (Some(command), Some(arg)) => {
                let command = command.to_lowercase();
                if command != "forward" && command != "down" && command != "up" {
                    return Err(ParseCommandError::Unknown(command.to_string()));
                }

                let arg = arg.parse::<usize>().map_err(ParseCommandError::ArgumentParseError)?;
                Ok(match command.as_str() {
                    "forward" => Self::Forward(arg),
                    "down" => Self::Down(arg),
                    "up" => Self::Up(arg),
                    _ => unreachable!(),
                })
            },
        }
    }
}

#[derive(Debug)]
struct Commands {
    /// The list of commands to execute
    commands: Vec<Command>,
}

trait State {
    /// Mutates the state by applying the `Command`
    fn mutate(&mut self, cmd: &Command);


    /// Get the `(horizontal position, depth)` tuple
    fn get(&self) -> (usize, usize);
}

#[derive(Default, Debug)]
struct BasicState {
    /// The horizontal position
    horizontal: usize,

    /// The depth
    depth: usize
}

impl State for BasicState {
    fn mutate(&mut self, cmd: &Command) {
        match cmd {
            Command::Forward(x) => self.horizontal += *x,
            Command::Down(x) => self.depth += *x,
            Command::Up(x) => self.depth -= *x
        }
    }
    
    fn get(&self) -> (usize, usize) {
        (self.horizontal, self.depth)
    }
}

#[derive(Default, Debug)]
struct AimingState {
    /// The aim
    aim: usize,

    /// The horizontal position
    horizontal: usize,

    /// The depth
    depth: usize
}

impl State for AimingState {
    fn mutate(&mut self, cmd: &Command) {
        match cmd {
            Command::Down(x) => self.aim += *x,
            Command::Up(x) => self.aim -= *x,
            Command::Forward(x) => {
                self.horizontal += x;
                self.depth += self.aim * x;
            }
        }
    }

    fn get(&self) -> (usize, usize) {
        (self.horizontal, self.depth)
    }
}

impl Commands {
    fn new(lines: Vec<String>) -> Result<Self, SolverError> {
        let commands = lines
            .iter()
            .map(|l| Command::from_str(l.as_str()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e|  SolverError::Generic(e.into()))?;

        Ok(Commands{
            commands
        })
    }

    fn execute_on(&self, state: &mut dyn State) {
        self.commands.iter().for_each(|c| state.mutate(c));
    }
}

struct Day2;

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day2)
}

fn solve<S: State + Default>(lines: Vec<String>) -> SolverResult {
    let commands = Commands::new(lines)?;
    let mut state = S::default();
    commands.execute_on(&mut state);

    let (horizontal, depth) = state.get();
    Ok((horizontal * depth).to_string())
}

impl Solver for Day2 {
    fn name(&self) ->  &'static str {
        "Dive!"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        solve::<BasicState>(lines)
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        solve::<AimingState>(lines)
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "150",
            2 => "900",
            _ => unreachable!()
        }
    }
}
