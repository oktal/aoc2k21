use super::{Solver, SolverError, SolverResult};

use std::collections::VecDeque;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
enum CaveTryFromError {
    Empty,
    /// The cave is invalid
    Invalid(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Cave {
    Entry,
    Exit,
    Small(String),
    Big(String),
}

impl FromStr for Cave {
    type Err = CaveTryFromError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(CaveTryFromError::Empty);
        }

        let valid = s.chars().all(|c| c.is_ascii_alphabetic());
        if !valid {
            return Err(CaveTryFromError::Invalid(s.to_string()));
        }

        let s_lower = s.to_lowercase();
        Ok(match s_lower.as_str() {
            "start" => Cave::Entry,
            "end" => Cave::Exit,
            _ => {
                let is_upper_case = s.chars().all(char::is_uppercase);
                if is_upper_case {
                    Cave::Big(s.to_string())
                } else {
                    Cave::Small(s.to_string())
                }
            }
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct EdgeIndex(usize);
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct NodeIndex(usize);

#[derive(Debug)]
struct Node {
    data: Cave,
    edge: Option<EdgeIndex>,
}

#[derive(Debug)]
struct Edge {
    target: NodeIndex,
    next: Option<EdgeIndex>,
}

#[derive(Debug)]
struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    fn new() -> Graph {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn find_node(&self, data: Cave) -> Option<NodeIndex> {
        self.nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.data == data)
            .map(|(idx, _)| NodeIndex(idx))
    }

    pub fn add_node(&mut self, data: Cave) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(Node { data, edge: None });

        NodeIndex(index)
    }

    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex) -> Option<EdgeIndex> {
        let index = self.edges.len();

        let _target_node = self.nodes.get(target.0)?;
        let source_node = self.nodes.get_mut(source.0)?;

        let edge = Edge {
            target,
            next: source_node.edge,
        };

        self.edges.push(edge);
        let edge_index = Some(EdgeIndex(index));

        source_node.edge = edge_index;
        edge_index
    }
}

/// A trait to determine the visiting rule for a given cave in the cave system
trait VisitRule {
    /// The path we are currently traversing
    type Path: Clone;

    /// Visit a node and return a new `Some(new_path)` if we can visit it or `None`
    fn visit(
        graph: &Graph,
        curent_path: &Self::Path,
        node_index: NodeIndex,
        node: &Node,
    ) -> Option<Self::Path>;

    /// Create a `Self::Path` from a `path`
    fn create_path(path: Vec<NodeIndex>) -> Self::Path;

    /// Get the `path` from a `Self::Path`
    fn get_path(path: Self::Path) -> Vec<NodeIndex>;

    /// Return the last element of `Self::Path`
    fn last(path: &Self::Path) -> NodeIndex;
}

struct VisitBigMultipleSmallOnce;

/// visit small caves at most once, and visit big caves any number of times.
impl VisitRule for VisitBigMultipleSmallOnce {
    type Path = Vec<NodeIndex>;

    fn visit(
        _graph: &Graph,
        current_path: &Self::Path,
        node_index: NodeIndex,
        node: &Node,
    ) -> Option<Self::Path> {
        let data = &node.data;

        // This is a big cave, we can visit it multiple times
        if let Cave::Big(_) = data {
            let mut new_path = current_path.clone();
            new_path.push(node_index);
            return Some(new_path);
        }

        // We can only visit the cave if we did not visit it already
        let already_visited = current_path.iter().find(|&&n| n == node_index);
        match already_visited {
            None => {
                let mut new_path = current_path.clone();
                new_path.push(node_index);
                Some(new_path)
            }
            Some(_) => None,
        }
    }

    fn create_path(path: Vec<NodeIndex>) -> Self::Path {
        path
    }

    fn get_path(path: Self::Path) -> Vec<NodeIndex> {
        path
    }

    fn last(path: &Self::Path) -> NodeIndex {
        path[path.len() - 1]
    }
}

struct VisitBigMultipleSingleSmallTwiceOtherOnce;

/// big caves can be visited any number of times, a single small cave can be visited at most twice,
/// and the remaining small caves can be visited at most once.
impl VisitRule for VisitBigMultipleSingleSmallTwiceOtherOnce {
    /// Our path with a boolean flag to know whether we already visited a small cave twice
    type Path = (bool, Vec<NodeIndex>);

    fn visit(
        _graph: &Graph,
        current_path: &Self::Path,
        node_index: NodeIndex,
        node: &Node,
    ) -> Option<Self::Path> {
        let data = &node.data;

        // This is a big cave, we can visit it multiple times
        if let Cave::Big(_) = data {
            let mut new_path = current_path.clone();
            new_path.1.push(node_index);
            return Some((new_path.0, new_path.1));
        }

        // How many times did we alredy visit this cave ?
        let n_visited = current_path.1.iter().filter(|&&n| n == node_index).count();

        let res = match (n_visited, current_path.0, data) {
            // We never visited that cave yet
            (0, _, _) => {
                let mut new_path = current_path.clone();
                new_path.1.push(node_index);
                Some((new_path.0, new_path.1))
            }

            // We already visited the entry or the exit, we can not visit it again
            (_, _, Cave::Entry | Cave::Exit) => None,

            // We already visited that cave once and we did not visit a small cave twice yet, so
            // can visit it a second time
            (1, false, _) => {
                let mut new_path = current_path.clone();
                new_path.1.push(node_index);
                Some((true, new_path.1))
            }

            _ => None,
        };

        res
    }

    fn create_path(path: Vec<NodeIndex>) -> Self::Path {
        (false, path)
    }

    fn get_path(path: Self::Path) -> Vec<NodeIndex> {
        path.1
    }

    fn last(path: &Self::Path) -> NodeIndex {
        path.1[path.1.len() - 1]
    }
}

#[derive(Debug)]
enum CaveError {
    /// The cave system is missing an entry
    MissingEntry,

    /// The cave system is missing an exit
    MissingExit,

    /// An invalid connection
    InvalidConnection,

    /// Invalid cave identifier
    InvalidCave(CaveTryFromError),
}

impl std::error::Error for CaveError {}

impl fmt::Display for CaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct CaveSystem {
    /// The graph representing the graph
    graph: Graph,

    /// The index of the cave's entry
    entry: NodeIndex,

    /// The index of the cave's exit
    exit: NodeIndex,
}

impl CaveSystem {
    fn parse(lines: Vec<String>) -> Result<CaveSystem, CaveError> {
        let mut graph = Graph::new();

        for line in lines {
            let (source, target) = Self::parse_line(line)?;

            let source_node = graph
                .find_node(source.clone())
                .unwrap_or(graph.add_node(source.clone()));
            let target_node = graph
                .find_node(target.clone())
                .unwrap_or(graph.add_node(target.clone()));

            graph.add_edge(source_node, target_node);
            graph.add_edge(target_node, source_node);
        }

        let entry = graph
            .find_node(Cave::Entry)
            .ok_or(CaveError::MissingEntry)?;
        let exit = graph.find_node(Cave::Exit).ok_or(CaveError::MissingExit)?;

        Ok(CaveSystem { graph, entry, exit })
    }

    fn parse_line(line: String) -> Result<(Cave, Cave), CaveError> {
        let mut split = line.split('-');

        let source = split.next().ok_or(CaveError::InvalidConnection)?;
        let target = split.next().ok_or(CaveError::InvalidConnection)?;

        let source = source.parse::<Cave>().map_err(CaveError::InvalidCave)?;
        let target = target.parse::<Cave>().map_err(CaveError::InvalidCave)?;

        Ok((source, target))
    }

    fn find_paths<V: VisitRule>(&self, start: NodeIndex, target: NodeIndex) -> Vec<Vec<NodeIndex>> {
        // The queue of possible paths
        let mut path_queue: VecDeque<V::Path> = VecDeque::new();

        // Enqueue the starting point
        path_queue.push_back(V::create_path(vec![start]));

        let mut paths = Vec::new();

        while let Some(current_path) = path_queue.pop_front() {
            // The last element of our current path
            let last = V::last(&current_path);

            // If we reached our destination, add the current path the list of paths and continue
            if last == target {
                paths.push(V::get_path(current_path));
                continue;
            }

            let last_node = &self.graph.nodes[last.0];
            let mut edge_index = last_node.edge;

            // Loop over the edges for the last node of our current path
            while let Some(index) = edge_index {
                let edge = &self.graph.edges[index.0];
                let target = edge.target;
                let target_node = &self.graph.nodes[target.0];

                if let Some(new_path) = V::visit(&self.graph, &current_path, target, &target_node) {
                    path_queue.push_back(new_path);
                }

                // Follow the link to the next edge
                edge_index = edge.next;
            }
        }

        paths
    }
}

struct Day12;

impl Solver for Day12 {
    fn name(&self) -> &'static str {
        "Passage Pathing"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        let cave_system = CaveSystem::parse(lines).map_err(|e| SolverError::Generic(e.into()))?;
        let paths = cave_system
            .find_paths::<VisitBigMultipleSmallOnce>(cave_system.entry, cave_system.exit);

        Ok(paths.len().to_string())
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        let cave_system = CaveSystem::parse(lines).map_err(|e| SolverError::Generic(e.into()))?;
        let paths = cave_system.find_paths::<VisitBigMultipleSingleSmallTwiceOtherOnce>(
            cave_system.entry,
            cave_system.exit,
        );

        Ok(paths.len().to_string())
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "226",
            2 => "3509",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day12)
}
