use std::{env, net::{IpAddr, Ipv4Addr, SocketAddr}, io::{self, stdin, stdout, Stdout, Write}, process::{self, exit}, sync::{Arc, Mutex, RwLock}, thread, time::Duration};
use crossterm::{cursor::MoveTo, event::{self, Event, EventStream, KeyCode, KeyModifiers}, terminal::{self, Clear, ClearType}, QueueableCommand};
use tokio::net::lookup_host;

use crate::{get_local, scanner};


// ---- TODO ---- some slightly more graceful error handling.

#[derive(Debug, Clone, PartialEq)]
pub enum ArbiterError {
    NotEnoughArguments,
    TooManyArguments,
    InvalidCommand,
    InvalidIpaddress,
    InvalidPort,
    InvalidFlag,
    
    UnknownInvalid,
    None
}


#[derive(Debug, Clone, PartialEq)]
pub enum CommandFlag {
    Range,
    Interface,
    Output
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandType {
    Scan
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub command: CommandType,
    pub flag_zero: Option<CommandFlag>,
    pub flag_one: Option<CommandFlag>,
    pub flag_two: Option<CommandFlag>,

    pub ip: String,
    pub port: Option<u16>,
    pub port_range: Option<Vec<u16>>,

    pub valid: bool
}

impl Command {
    pub fn default() -> Self {
        Self {
            command: CommandType::Scan,
            flag_zero: Some(CommandFlag::Range),
            flag_one: None,
            flag_two: None,

            ip: "127.0.0.1".to_string(),
            port: Some(12345),
            port_range: None,

            valid: false
        }
    }/* match command {
        
    } */
}

// lex arguments works with a trimmed set of arguments, the executable path is cut off by the return_args() function.
pub async fn lex_arguments(args: &Vec<String>) -> Command  {
    let mut command: Command = Command::default();
    let localhost = get_local::get_localhost();

    match args.len() {

        // ---------------- invalid ------------------

        1 | 2 => {
            exit_with_error(ArbiterError::NotEnoughArguments, None);
            command
        }

        // --------------- 3 arguments ---------------

        3 => { // <command> <ip> <port> should just scan 1 port.

            match args.get(0).unwrap().to_lowercase().as_str() { // for some reason cant match a string literal with .to_string()
                "scan" => {
                    command.command = CommandType::Scan;
                    command.valid = true;
                }
                _ => {
                    println!("{:?}", command.command.clone()); //dbg
                    exit_with_error(ArbiterError::InvalidCommand, Some(format!("{}", args.get(0).unwrap())));
                }
            }

            command.flag_zero = None;
            command.ip = args[1].clone();
            command.port = Some(args[2].parse().clone().unwrap());
            command
        }

        // ---------------------- 4 arguments ---------------

        4 | 5 => { // <command> <ip> <flag> <port> <maybe end port>

            let arg_command = args.get(0).unwrap();
            let arg_ip = args.get(1).unwrap();
            let arg_flag = args.get(2).unwrap();
            let arg_port = args.get(3).unwrap();
            let arg_end_port = args.get(4).unwrap(); 

            match arg_command.to_lowercase().as_str() {
                "scan" => {
                    command.command = CommandType::Scan;
                    command.valid = true;
                }
                _ => {
                    println!("{:?}", command.command.clone()); // dbg
                    exit_with_error(ArbiterError::InvalidCommand, Some(format!("{}", args.get(0).unwrap())));
                }
            } 
            match host_exists(arg_ip.to_string(), localhost.unwrap()).await  { 
                true => { } false => {
                    exit_with_error(ArbiterError::InvalidIpaddress, Some(args.get(1).unwrap().to_owned())) } }
            
            let mut flags: Vec<CommandFlag> = Vec::new();

            for character in arg_flag.chars() {
                match character {
                    '-' => {
                        // this is just the flag...character.
                    }
                    'r' | 'R' => {
                        flags.push(CommandFlag::Range);
                    }
                    'i' | 'I' => {
                        flags.push(CommandFlag::Interface);
                    }
                    _ => {
                        exit_with_error(ArbiterError::InvalidFlag, Some(character.to_string().clone()))
                    }
                }
            }
            for element in flags {
                match element {
                    CommandFlag::Range => {
                        command.flag_zero = Some(CommandFlag::Range)
                    }
                    CommandFlag::Interface => {
                        command.flag_one = Some(CommandFlag::Interface)
                    }
                    _ => {
                        print_error(ArbiterError::None, Some("ERROR: If you're trying to ouput something to a file, specify '-o' at the end. EG:
arbiter scan localhost -r 2000 2005 -o file.txt".to_string()))
                    }
                }
            }

            let port_range: Vec<u16> = (args[3].parse::<u16>().unwrap()..args[4].parse::<u16>().unwrap() + 1).collect();


            command.ip = args[1].clone();
            command.port = Some(args[3].parse().unwrap()); // start port
            command.port_range = Some(port_range);

            command
        }
        
        // anything else ??

        _ => {
            println!("{:?}", args);
            kill("Something has gone terribly, terribly wrong.");
            command
        }
    }
}

async fn host_exists(host: String, localhost: IpAddr) -> bool {
    if localhost.to_string() == host { // returns true if host is localhost, else lookup host
        return true;
    } else { }
    match lookup_host(host).await {
        Ok(mut addrs) => {
            if addrs.next().is_some() {
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

fn parse_integer(string: &str) -> bool {
    string.chars().all(|c| c.is_digit(10))
}

// ------------- public functions ---------------

pub fn print_error(error: ArbiterError, mut msg: Option<String>) { // lots of boilerplat :(
    if msg.is_some() {} else {msg = Some("<error: no messsage>".to_string())}
    match error {
        ArbiterError::NotEnoughArguments => {
            eprintln!("ERROR: Not enough arguments.");
        }
        ArbiterError::TooManyArguments => {
            eprintln!("ERROR: Too many arguments.");
        }
        ArbiterError::InvalidCommand => {
            eprintln!("ERROR: Command {} does not exist.", msg.unwrap())
        }
        ArbiterError::InvalidIpaddress => {
            if msg.is_some() {
                eprintln!("ERROR: Could not lookup host IP: {:?}.", msg.unwrap())
            } else {
                eprintln!("ERROR: Could not lookup host IP.")
            }
        }
        ArbiterError::InvalidPort => {
            if msg.is_some() {
                eprintln!("ERROR: Invalid port value: {:?}", msg.clone().unwrap())
            } else {
                eprintln!("ERROR: Invalid port value.")
            }println!("ERROR: Invalid port value: {:?}", msg.clone().unwrap())
        }
        ArbiterError::InvalidFlag => {
            eprintln!("ERROR: Invalid flag(s).")
        }
        ArbiterError::UnknownInvalid => {
            eprintln!("ERROR: Unknown errror.")
        }
        _ => {

        }
    }
}


pub fn exit_with_error(error: ArbiterError, mut msg: Option<String>) {
    if msg.is_some() {} else {msg = Some("<error: no message>".to_string())}

    match error {
        ArbiterError::NotEnoughArguments => {
            eprintln!("ERROR: Not enough arguments.");
            process::exit(1);
        }
        ArbiterError::TooManyArguments => {
            eprintln!("ERROR: Too many arguments.");
            process::exit(1);
        }
        ArbiterError::InvalidCommand => {
            eprintln!("ERROR: Command {} does not exist.", msg.unwrap())
        }
        ArbiterError::InvalidIpaddress => {
            if msg.is_some() {
                eprintln!("ERROR: Could not lookup host IP: {:?}.", msg.unwrap())
            } else {
                eprintln!("ERROR: Could not lookup host IP.");
            }
            process::exit(1);
        }
        ArbiterError::InvalidPort => {
            eprintln!("ERROR: Invalid port value.");
            process::exit(1);
        }
        ArbiterError::InvalidFlag => {
            eprintln!("ERROR: Invalid flag");
            process::exit(1);
        }
        ArbiterError::UnknownInvalid => {
            eprintln!("ERROR: Unknown Error");
            process::exit(1);
        }
        _ => {
            println!("Something has gone terribly terribly wrong. This should never happen.")
        }
    }
}

pub fn return_args(mut args: Vec<String>) -> Vec<String> {
    if !args.is_empty() {
        args.remove(0);
    } else {
        kill("Zero arguments somehow xD")
    }

    args
}

pub fn initialize() -> Stdout {
    let mut stdout = stdout();
    let _ = terminal::enable_raw_mode();
    stdout.queue(Clear(ClearType::All)).unwrap();
    stdout.flush().unwrap();

    stdout
}

pub fn kill(msg: &str) {
    let mut stdout = stdout();
    let _ = terminal::disable_raw_mode();
    let _ = stdout.queue(Clear(ClearType::All));
    let _ = stdout.flush();
    
    println!("ERROR: {}", msg);
    process::exit(1);
} 

// ------------------ runner -----------------

pub async fn runner(command: Command) {
    if command.valid {} else {exit_with_error(ArbiterError::UnknownInvalid, Some("Command not valid for some reason".to_string())) }
}