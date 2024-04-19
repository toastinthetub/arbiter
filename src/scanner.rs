use std::io::{self, Write};
use tokio::net::TcpStream;
use tokio::time::{self, Duration};
use crossterm::{terminal::{self, Clear, ClearType}, cursor::MoveTo, event::{self, Event, EventStream, KeyCode, KeyModifiers}};

pub async fn scan_ports(ip: String, ports: Vec<u16>) {
    for port in ports {
        scan_port(ip.clone(), port).await;
    }
}

pub async fn scan_port(addr: String, port: u16) {
    match time::timeout(Duration::from_secs(5), TcpStream::connect((addr.clone(), port))).await {
        Ok(Ok(_)) => {
            println!("Port {} is open", port)
        }
        _ => {
            println!("Port {} is closed", port)
        }
    }
}