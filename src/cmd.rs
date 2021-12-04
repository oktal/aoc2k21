use std::vec::Vec;
use std::string::String;
use std::str::FromStr;

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(super) enum ParsePathError {
    Empty,

    InvalidPath(PathBuf),
    InvalidIndex(String, std::num::ParseIntError)
}

#[derive(Debug, Eq, PartialEq)]
struct ArgPathFragment {
    prefix: String,
    index: Option<usize>,
}

impl ArgPathFragment {
    fn parse(s: &str) -> std::result::Result<ArgPathFragment, ParsePathError> {
        if s.is_empty() {
            return Err(ParsePathError::Empty)
        }

        let (prefix, index) = match s.find(|c: char| c.is_ascii_digit()) {
            Some(idx) => {
                let (prefix, index) = s.split_at(idx);
                let index = index.parse::<usize>().map_err(|e| ParsePathError::InvalidIndex(s.into(), e))?;

                (prefix, Some(index))
            },
            None => (s, None)
        };

        Ok(ArgPathFragment{
            prefix: prefix.into(),
            index
        })

    }
}

#[derive(Debug)]
struct ArgPath {
    value: String,
    fragments: Vec<ArgPathFragment>
}

impl ArgPath {
    fn parse(s: &str) -> std::result::Result<Self, ParsePathError> {
        if s.is_empty() {
            return Err(ParsePathError::Empty);
        }

        let fragments = s
            .split('/')
            .map(ArgPathFragment::parse)
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(ArgPath{
            value: s.into(),
            fragments
        })
    }

    fn parse_path<P: AsRef<Path>>(path: P) -> std::result::Result<Self, ParsePathError> {
        let file_name = path
            .as_ref()
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or(
                ParsePathError::InvalidPath(PathBuf::from(path.as_ref()))
            )?;

        let mut file_parts: Vec<_> = file_name
            .split('.')
            .collect();

        // Remove the extension from the file name
        file_parts.pop();

        let fragments = file_parts
            .into_iter()
            .map(ArgPathFragment::parse)
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(ArgPath{
            value: file_name.to_string(),
            fragments
        })
    }

    fn disjoint(&self, other: &ArgPath) -> Option<&ArgPathFragment> {
        for i in 0..self.fragments.len() {
            if i >= other.fragments.len() {
                return Some(&self.fragments[i]);
            }
            else if self.fragments[i] != other.fragments[i] {
                return Some(&self.fragments[i])
            }
        }

        None
    }

}

impl FromStr for ArgPath {
    type Err = ParsePathError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        ArgPath::parse(s)
    }

}

#[derive(Debug)]
pub(super) struct CommonArgs {
    path: ArgPath
}

#[derive(Debug)]
pub(super) enum Error {
    MissingCommand,
    MissingPath(String),

    InvalidCommand(String),
    InvalidPath(ParsePathError),

    ReadInputDirectory(PathBuf, std::io::Error),
}

#[derive(Debug)]
pub(super) enum Command {
    Solve(CommonArgs),
    Test(CommonArgs),
}

pub(super) type Result<T> = std::result::Result<T, Error>;

fn read_input_files<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>> {
    let mut input_files: Vec<PathBuf> = Vec::new();

    let entry_iter = fs::read_dir(path.as_ref()).map_err(
        |e|
            Error::ReadInputDirectory(
                PathBuf::from(path.as_ref()),
                e
            )
        )?;

    for entry in entry_iter {
        let entry = entry.map_err(
            |e|
                Error::ReadInputDirectory(
                    PathBuf::from(path.as_ref()),
                    e
                )
            )?;
        let path = entry.path();

        if path.is_file() {
            input_files.push(path.into())
        }
    }

    Ok(input_files)
}

impl Command {
    pub(super) fn parse_from_args() -> Result<Self> {
        let args = std::env::args()
            .skip(1)
            .collect::<Vec<_>>();
        Self::parse(args)
    }

    fn parse(args: Vec<String>) -> Result<Self> {
        let command = args.get(0).ok_or(Error::MissingCommand)?;
        let command = command.to_lowercase();

        let is_valid = matches!(command.as_str(), "test" | "solve");
        if !is_valid {
            return Err(Error::InvalidCommand(command));
        }

        let path = args
            .get(1)
            .ok_or(Error::MissingPath(command.clone()))
            .and_then(
                |p|
                    ArgPath::from_str(p.as_str()).map_err(Error::InvalidPath)
            )?;

        let args = CommonArgs{path};
        Ok(match command.as_str() {
            "test" => Command::Test(args),
            "solve" => Command::Solve(args),
            _ => unreachable!()

        })
    }

    fn args(&self) -> &CommonArgs  {
        match self {
            Self::Solve(args) | Self::Test(args) => args,
        }
    }

    pub(super) fn resolve_input_files<P: AsRef<Path>>(&self, prefix_path: P) -> Result<Vec<PathBuf>> {
        let args = self.args();

        let mut input_files = Vec::new();

        let is_test = matches!(self, Self::Test(_));

        let files = read_input_files(prefix_path)?;
        for file in &files {
            let file_path = ArgPath::parse_path(&file).map_err(Error::InvalidPath)?;

            if let Some(fragment) = file_path.disjoint(&args.path) {
                if fragment.prefix == "test" && is_test {
                    input_files.push(file.to_path_buf());
                }
            } else {
                input_files.push(file.to_path_buf());
            }
        }

        Ok(input_files)
    }
}
