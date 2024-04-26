use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;
use std::str;

pub fn get_localhost() -> Option<IpAddr> {
    let output = Command::new("ifconfig")
        .output()
        .expect("failed to execute `ifconfig`");

    let stdout = str::from_utf8(&output.stdout)
        .expect("failed to parse ifconfig output");

    for line in stdout.lines() {
        if let Some(host) = extract_ip_address(line) {
            if host != "127.0.0.1" {
                if let Ok(addr) = host.parse::<Ipv4Addr>() {
                    return Some(IpAddr::V4(addr));
                }
                // Check for IPv6 addresses if needed
            }
        }
    }

    None
}

fn extract_ip_address(line: &str) -> Option<&str> {
    let mut parts = line.trim().split_whitespace();

    if parts.next() == Some("inet") {
        parts.next().and_then(|part| {
            if part.starts_with("addr:") {
                Some(&part[5..])
            } else {
                Some(part)
            }
        })
    } else {
        None
    }
}
