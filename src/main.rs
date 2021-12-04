mod cmd;

use cmd::Command;

const INPUT_PREFIX: &'static str = "inputs";

fn handle_command(command: Command) -> cmd::Result<()> {
    println!("Handling {:#?}", command);
    
    let input_files = command.resolve_input_files(INPUT_PREFIX);
    println!("input_files: {:?}", input_files);

    Ok(())
}


fn main() {
    Command::parse_from_args()
        .and_then(handle_command)
        .expect("Failed to handle command");
}
