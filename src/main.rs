mod scanner;
mod utils;

mod get_local;

use std::{env, io::{stdout, stdin}};
use crossterm::{terminal::{self, Clear, ClearType}, cursor::MoveTo, event::{self, Event, EventStream, KeyCode, KeyModifiers}};
use utils::{exit_with_error, initialize, lex_arguments, return_args, CommandFlag};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let args = return_args(args);
    let command: utils::Command  =  lex_arguments(&args).await;

    match command.valid {
        true => {
            match command.command {
                utils::CommandType::Scan => {
                    if command.flag_zero == Some(CommandFlag::Range) && command.port_range.is_some() {
                        scanner::scan_ports(command.ip.clone(), command.port_range.unwrap()).await;
                    } else if command.flag_zero == None {
                        scanner::scan_port(command.ip.clone(), command.port.unwrap()).await;
                    }
                }
            }
        } false => {
            exit_with_error(utils::ArbiterError::UnknownInvalid, None)
        }
    }
}