use std::{env, io::{self, stdin, stdout, Stdout, Write}, os::unix::process, process::exit, sync::{Arc, Mutex, RwLock}, thread, time::Duration};
use crossterm::{cursor::MoveTo, event::{self, Event, EventStream, KeyCode, KeyModifiers}, terminal::{self, Clear, ClearType}, QueueableCommand};
use tokio::net::lookup_host;

use crate::scanner;


const TERMINAL_ERROR: &str = "ERROR: Terminal Error";
const NETWORK_ERROR: &str = "ERROR: Network Error";
const BAD_ARGS_A: &str = "ERROR: Invalid Arguments: Not enough arguments";
const BAD_ARGS_B: &str = "ERROR: Invalid Arguments";

const DIVINE_INTERVENTION: &str = "ERROR: Bad arguments...of some sort";

// ---- TODO ---- some slightly more graceful error handling.

enum Command {
    scan_port,
    scan_ports
}

pub async fn lex_arguments(args: &Vec<String>) {
    match args.len() {
        1 | 2 => {
            eprint!("{}", BAD_ARGS_A);
            exit(0);
        }

        3 | 4 => {
            let mut integers: Vec<u16> = Vec::new();

            for element in args {
                let valid = parse_integer(&element);
                if valid {
                    integers.push(element.parse().expect(BAD_ARGS_B));
                } else {}
            }

            println!("{:?}", integers);
            println!("{:?}", args[1].clone());

            if integers.len() == 1 {
                let mut port_range: Vec<u16> = Vec::new();
                port_range.push(1);
                scanner::scan_ports(integers[1].to_string(), port_range);
            } else if integers.len() == 2 {
                let mut port_range: Vec<u16> = Vec::new();
                for port in &mut integers[1..2] {
                    println!("did we get here? B");
                    port_range.push(*port);
                    scanner::scan_ports(args[1].clone(), port_range.clone())
                }
            }
        }

        _ => {
            println!("we fuckin got here.");
            eprint!("{}", DIVINE_INTERVENTION);
            exit(0);
        }
    }
} 

async fn check_host_exists(ip: String) -> bool {
    match lookup_host(ip).await {
        Ok(_) => true,
        Err(_) => false
    }
}

fn parse_integer(string: &str) -> bool {
    string.chars().all(|c| c.is_digit(10))
}

pub fn initialize() -> Stdout {
    let mut stdout = stdout();
    let _ = terminal::enable_raw_mode();
    stdout.queue(Clear(ClearType::All)).expect(TERMINAL_ERROR);
    stdout.flush().expect(TERMINAL_ERROR);

    stdout
}
