// @todo A lot of this was shamelessly nicked from z - we should really make it its own crate.
#![allow(unused_imports)]

use anyhow::{anyhow, Result};
use libc;
use std::collections::HashMap;
use std::env;
use std::net::TcpListener;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdout};
use tokio::process::{Child, Command};

/// Is this port in use?
pub fn is_port_available(port: u16) -> bool {
    //  TcpListener::bind(("127.0.0.1", port)).is_ok()
    TcpListener::bind(("0.0.0.0", port)).is_ok()
}

/// Find the next port up .. stop looking after range.
pub fn find_available_port(from: u16, range: u16) -> Result<u16> {
    for port in from..(from + range) {
        if is_port_available(port) {
            return Ok(port);
        }
    }
    Err(anyhow!("Ran out of ports to search; none is available"))
}

/// Find the next contiguous port range port up .. stop looking after range.
pub fn find_available_ports(from: u16, range: u16, ports_required: u16) -> Result<u16> {
    for port in from..(from + range) {
        let mut avail = true;
        for incr in 0..ports_required {
            if !is_port_available(port + incr) {
                avail = false;
                break;
            }
        }
        if avail {
            return Ok(port);
        }
    }
    Err(anyhow!("Ran out of ports to search; none is available"))
}
