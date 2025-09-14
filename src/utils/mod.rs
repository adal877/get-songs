use colored::*;

pub fn println_err(msg: &str) {
    eprintln!("{}", msg.bright_red());
}

pub fn println_success(msg: &str) {
    eprintln!("{}", msg.bright_green());
}

pub fn println_alert(msg: &str) {
    eprintln!("{}", msg.bright_yellow());
}

