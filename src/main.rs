mod scanner;
mod utils;

use std::{env, io::{stdout, stdin}};
use crossterm::{terminal::{self, Clear, ClearType}, cursor::MoveTo, event::{self, Event, EventStream, KeyCode, KeyModifiers}};
use utils::{initialize, lex_arguments};

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();

    lex_arguments(&args).await;
    
}
