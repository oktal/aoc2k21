use std::vec::Vec;
use std::string::String;

trait Solver {
    fn solve(&self, lines: Vec<String>);
}
