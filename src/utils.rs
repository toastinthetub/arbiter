use std::{env, net::{IpAddr, Ipv4Addr, SocketAddr}, io::{self, stdin, stdout, Stdout, Write}, process::{self, exit}, sync::{Arc, Mutex, RwLock}, thread, time::Duration};
use crossterm::{cursor::MoveTo, event::{self, Event, EventStream, KeyCode, KeyModifiers}, terminal::{self, Clear, ClearType}, QueueableCommand};
use tokio::net::lookup_host;

use crate::scanner;


// ---- TODO ---- some slightly more graceful error handling.

#[derive(Debug, Clone)]
pub enum ArbiterError {
    NotEnoughArguments,
    TooManyArguments,
    InvalidCommand,
    InvalidIpaddress,
    InvalidPort,
    InvalidFlag,
    
    UnknownInvalid
}


#[derive(Debug, Clone)]
pub enum CommandFlag {
    Range,
    Interface,
    Output
}

#[derive(Debug, Clone)]
pub enum CommandType {
    Scan
}

#[derive(Debug, Clone)]
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
    let localhost = get_localhost().await;

    println!("{:?}", args.len());

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
            match args.get(0).unwrap().to_lowercase().as_str() {
                "scan" => {
                    command.command = CommandType::Scan;
                    command.valid = true;
                }
                _ => {
                    println!("{:?}", command.command.clone()); // dbg
                    exit_with_error(ArbiterError::InvalidCommand, Some(format!("{}", args.get(0).unwrap())));
                }
            }

            let exists = host_exists(args.get(1.clone()).to_owned().unwrap().to_string(), localhost.unwrap()).await;
            if exists { /*  woohoo! */ } else {
                exit_with_error(ArbiterError::InvalidIpaddress, Some(args.get(1).unwrap().to_owned())) }
            

            let port_range: Vec<u16> = (args[3].parse::<u16>().unwrap()..args[4].parse::<u16>().unwrap() + 1).collect();


            command.flag_zero = None; // this should change soon 
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

async fn get_localhost() -> Result<IpAddr, Box<dyn std::error::Error>> {
    let local_hostname = std::env::var("HOSTNAME")
        .unwrap_or_else(|_| "localhost".to_string());

    let ip_addrs = tokio::net::lookup_host(local_hostname)
        .await?;

    for addr in ip_addrs {
        if let IpAddr::V4(ipv4) = addr.ip() {
            if ipv4.is_loopback() {
                return Ok(IpAddr::V4(ipv4));
            }
        }
    }

    Err("Localhost not found".into())
}

fn parse_integer(string: &str) -> bool {
    string.chars().all(|c| c.is_digit(10))
}

// ------------- public functions ---------------

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
                eprintln!("ERROR: Could not lookup host IP: {:?}", msg.unwrap())
            } else {
                eprintln!("ERROR: Could not lookup host IP");
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