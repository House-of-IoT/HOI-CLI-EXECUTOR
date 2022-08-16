use crate::parser::accepted_commands;
use ansi_term::Colour;

pub fn clear_terminal() {
    print!("\x1B[2J\x1B[1;1H");
}

pub fn log_unknown_command(data: &str) {
    println!("[{}]:{}", Colour::Red.paint("INVALID_COMMAND"), data);
}

pub fn log_event(data: &str) {
    println!("[{}]:{}", Colour::Purple.paint("EVENT"), data);
}

pub fn log_command(data: &str) {
    println!("[{}]:{}", Colour::Green.paint("COMMAND"), data);
}

pub fn log_command_parameters(data: &Vec<String>) {
    println!(
        "[{}]:[{:?}]",
        Colour::Green.paint("COMMAND Parameters"),
        data
    );
}

pub fn log_custom_green(data: &str, name_of_log: &str) {
    println!("[{}]:[{}]", Colour::Green.paint(name_of_log), data);
}

pub fn log_accepted_commands() {
    clear_terminal();
    println!("ALL ACCEPTED COMMANDS");
    let command_data = accepted_commands();
    let commands = command_data.iter();
    for command in commands {
        log_command(&*command);
    }
    log_command("help()");
    log_command("clear()");
    log_command("connection()");
    println!(" ");
}

pub fn log_current_connection_status(connection_str: &str, name: &str, outside_name: &str) {
    log_custom_green(connection_str, "Connection String");
    log_custom_green(name, "Your Name On Server:");
    log_custom_green(outside_name, "Server's Name")
}
