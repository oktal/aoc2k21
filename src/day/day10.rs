use super::{Solver, SolverError, SolverResult};

use std::convert::TryFrom;
use std::fmt::{self, Write};
use std::iter::Iterator;

use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum TokenKind {
    Opening,
    Closing,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Token {
    /// An opening (
    OpeningParenthesis,

    /// A closing )
    ClosingParenthesis,

    /// An opening [
    OpeningSquareBracket,

    /// A closing ]
    ClosingSquareBracket,

    /// An opening {
    OpeningBracket,

    /// A closing }
    ClosingBracket,

    /// An opening <
    OpeningAngleBracket,

    /// A closing >
    ClosingAngleBracket,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpeningParenthesis => f.write_char('('),
            Self::ClosingParenthesis => f.write_char(')'),
            Self::OpeningSquareBracket => f.write_char('['),
            Self::ClosingSquareBracket => f.write_char(']'),
            Self::OpeningBracket => f.write_char('{'),
            Self::ClosingBracket => f.write_char('}'),
            Self::OpeningAngleBracket => f.write_char('<'),
            Self::ClosingAngleBracket => f.write_char('>'),
        }
    }
}

#[derive(Debug)]
enum SyntaxError {
    InvalidToken(char),

    InvalidClosing { got: Token, expected: Token },
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxError::InvalidClosing { got, expected } => {
                write!(f, "Expected {}, but found {} instead", expected, got)
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

impl std::error::Error for SyntaxError {}

impl Token {
    fn closing(&self) -> Token {
        match self {
            Token::OpeningParenthesis => Token::ClosingParenthesis,
            Token::OpeningSquareBracket => Token::ClosingSquareBracket,
            Token::OpeningBracket => Token::ClosingBracket,
            Token::OpeningAngleBracket => Token::ClosingAngleBracket,
            token => *token,
        }
    }

    fn kind(&self) -> TokenKind {
        match self {
            Token::OpeningParenthesis
            | Token::OpeningSquareBracket
            | Token::OpeningBracket
            | Token::OpeningAngleBracket => TokenKind::Opening,

            Token::ClosingParenthesis
            | Token::ClosingSquareBracket
            | Token::ClosingBracket
            | Token::ClosingAngleBracket => TokenKind::Closing,
        }
    }
}

impl TryFrom<char> for Token {
    type Error = SyntaxError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '(' => Ok(Token::OpeningParenthesis),
            ')' => Ok(Token::ClosingParenthesis),
            '[' => Ok(Token::OpeningSquareBracket),
            ']' => Ok(Token::ClosingSquareBracket),
            '{' => Ok(Token::OpeningBracket),
            '}' => Ok(Token::ClosingBracket),
            '<' => Ok(Token::OpeningAngleBracket),
            '>' => Ok(Token::ClosingAngleBracket),
            _ => Err(SyntaxError::InvalidToken(c)),
        }
    }
}

struct Tokenizer<I: Iterator<Item = char>> {
    chars: I,
}

impl<I: Iterator<Item = char>> Iterator for Tokenizer<I> {
    type Item = Result<Token, SyntaxError>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_char = self.chars.next()?;
        Some(next_char.try_into())
    }
}

struct Line {
    _tokens: Vec<Token>,
    chunks: Vec<Token>,
}

impl FromStr for Line {
    type Err = SyntaxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokenizer = Tokenizer { chars: s.chars() };

        let tokens = tokenizer.collect::<Result<Vec<_>, _>>()?;
        let mut chunks = Vec::new();

        for token in &tokens {
            let token = token;
            match token.kind() {
                TokenKind::Opening => chunks.push(*token),
                TokenKind::Closing => {
                    let opening_token = chunks.pop();
                    if let Some(opening_token) = opening_token {
                        let expected_closing = opening_token.closing();
                        if expected_closing != *token {
                            return Err(SyntaxError::InvalidClosing {
                                expected: expected_closing,
                                got: *token,
                            });
                        }
                    }
                }
            }
        }

        Ok(Line {
            _tokens: tokens,
            chunks,
        })
    }
}

struct Day10;

impl Solver for Day10 {
    fn name(&self) -> &'static str {
        "Syntax Scoring"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        let mut score = 0u64;

        for line in lines {
            let line = line.parse::<Line>();
            if let Err(e) = line {
                if let SyntaxError::InvalidClosing { got, .. } = e {
                    score += match got {
                        Token::ClosingParenthesis => 3,
                        Token::ClosingSquareBracket => 57,
                        Token::ClosingBracket => 1197,
                        Token::ClosingAngleBracket => 25137,
                        _ => unreachable!(),
                    };
                } else {
                    return Err(SolverError::Generic(e.into()));
                }
            }
        }

        Ok(score.to_string())
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        let incomplete_lines = lines
            .iter()
            .map(|l| Line::from_str(&l))
            .filter(|l| l.is_ok())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SolverError::Generic(e.into()))?;

        let mut scores = Vec::new();

        for incomplete_line in incomplete_lines {
            let complete_tokens = incomplete_line.chunks.iter().rev().map(|t| t.closing());

            let score = complete_tokens.fold(0u64, |acc, token| {
                let mut score = acc * 5;
                score += match token {
                    Token::ClosingParenthesis => 1,
                    Token::ClosingSquareBracket => 2,
                    Token::ClosingBracket => 3,
                    Token::ClosingAngleBracket => 4,
                    _ => unreachable!(),
                };

                score
            });

            scores.push(score);
        }

        scores.sort();
        let median = scores.len() / 2;

        scores
            .get(median)
            .ok_or(SolverError::Generic("Failed to determine score".into()))
            .map(|s| s.to_string())
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "26397",
            2 => "288957",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day10)
}
