use super::{Solver, SolverError, SolverResult};

use std::result::Result;
use std::vec::Vec;

struct Day6;

#[derive(Debug)]
struct LanternFish {
    timer: u64
}

const FISHY: usize = 409664;

#[derive(Debug)]
struct BinaryLanternFish {
    timers: [u8; FISHY],

    len: usize,
}

#[derive(Debug)]
struct Spawns {
    timers: [u8; FISHY],

    index: usize,
}


impl Spawns {
    fn new() -> Spawns {
        Spawns { timers: [0u8; FISHY], index: 0 }
    }

    fn add(&mut self, timer: u8) -> Option<()> {
        if self.is_full()  {
            return None;
        }

        self.timers[self.index] = timer;
        self.index += 1;
        Some(())
    }

    fn merge_with(&mut self, other: Spawns) -> Option<Spawns> {
        if self.is_full() {
            return None;
        }

        let other_idx = other.index;
        let mut cur_idx = 0;

        let mut new_spawns = Spawns::new();

        while cur_idx != other_idx {
            let other = other.timers[cur_idx];

            if self.is_full() {
                new_spawns.add(other);
            } else {
                self.add(other);
            }

            cur_idx += 1;
        }

        if new_spawns.is_empty() {
            None
        } else {
            Some(new_spawns)
        }
    }


    fn is_empty(&self) -> bool {
        self.index == 0
    }

    fn is_full(&self) -> bool {
        self.index == FISHY
    }

    fn into_fish(self) -> BinaryLanternFish {
        BinaryLanternFish {
            timers: self.timers,
            len: self.index
        }
    }
}

impl BinaryLanternFish {
    fn from(timers: &[u8]) -> BinaryLanternFish {
        BinaryLanternFish { timers: timers.try_into().unwrap(), len: timers.len() }
    }
    
    fn count(&self) -> usize {
        self.len
    }

    fn spawn(&mut self) -> Option<Spawns> {
        let mut spawns = Spawns::new();
        for timer in self.timers.iter_mut().take(self.len) {
            if *timer > 0 {
                *timer -= 1;
            } else {
                *timer = FISH_RESET_TIMER as u8;
                spawns.add(NEW_FISH_TIMER as u8);
            }
        }

        if spawns.is_empty() {
            None
        } else {
            Some(spawns)
        }
    }

    fn print(&self) {
        for (idx, timer) in self.timers.iter().take(self.len).enumerate() {
            if idx > 0 {
                print!(",");
            }
            print!("{}", timer);
        }
    }
}

const NEW_FISH_TIMER: u64 = 8;
const FISH_RESET_TIMER: u64 = 6;

impl LanternFish {
    pub fn with_timer(timer: u64) -> LanternFish {
        LanternFish {  timer }
    }

    pub fn spawn(&mut self) -> Option<LanternFish> {
        if self.timer == 0 {
            self.timer = FISH_RESET_TIMER;
            Some(LanternFish::with_timer(NEW_FISH_TIMER))
        } else {
            self.timer -= 1;
            None
        }
    }
}

fn solve_v1(lines: Vec<String>, days: usize) -> SolverResult {
    let mut fishes = lines[0]
        .split(',')
        .map(|s| {
            s.parse::<u64>().map(|x| LanternFish::with_timer(x))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SolverError::Generic(e.into()))?;

    (0..days).for_each(|d| {
        let new_fishes: Vec<_> = 
            fishes
            .iter_mut()
            .filter_map(|f|  f.spawn())
            .collect();
        fishes.extend(new_fishes);
    });

    Ok(fishes.len().to_string())
}

fn solve_v2(lines: Vec<String>, days: usize) -> SolverResult {
    let values  = lines[0]
        .split(',')
        .map(|s| {
            s.parse::<u8>()
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SolverError::Generic(e.into()))?;

    let mut fishes = Vec::new();

    let mut cur_spawns = Spawns::new();
    for value in values {
       cur_spawns.add(value);
    }

    fishes.push(cur_spawns.into_fish());

    for day in 0..days {
        println!("day {}", day);
        let mut new_fishes = Vec::new();

        let mut cur_spawns = Spawns::new();

        for fish in &mut fishes {
            if let Some(spawns) = fish.spawn() {
                if cur_spawns.is_full() {
                    new_fishes.push(cur_spawns.into_fish());
                    cur_spawns = Spawns::new();
                }

                if let Some(new_spawns) = cur_spawns.merge_with(spawns) {
                    new_fishes.push(cur_spawns.into_fish());
                    cur_spawns = new_spawns;
                }
            }
        }

        if !cur_spawns.is_empty() {
            fishes.push(cur_spawns.into_fish());
        }

        fishes.extend(new_fishes);
        println!("Vec size is {}", fishes.len());
    }

    print!("\n");

    let total = fishes.iter().map(|f| f.count()).sum::<usize>();
    Ok(total.to_string())
}

impl Solver for Day6 {
    fn name(&self) -> &'static str {
        "Lanternfish"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        solve_v1(lines, 80)
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        solve_v2(lines, 256)
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "5934",
            2 => "26984457539",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day6)
}
