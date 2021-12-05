mod cmd;
mod day;

use cmd::Command;

const INPUT_PREFIX: &'static str = "inputs";

fn main() {
    Command::parse_from_args()
        .and_then(|c| c.run(INPUT_PREFIX))
        .expect("Failed to handle command");
}
