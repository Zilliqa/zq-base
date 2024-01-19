// @todo A lot of this was shamelessly nicked from z - we should really make it its own crate.
#![allow(unused_imports)]

use anyhow::{anyhow, Result};
use home;
use libc;
use std::collections::HashMap;
use std::env;
use std::net::TcpListener;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdout};
use tokio::process::{Child, Command};

/// Get a string from a u8
pub fn string_or_empty_from_u8(in_val: &[u8]) -> String {
    let result: &str = if let Ok(val) = std::str::from_utf8(in_val) {
        val
    } else {
        "<not_representable>"
    };
    result.to_string()
}

/// Get string from path
pub fn string_from_path(in_path: &Path) -> Result<String> {
    Ok(in_path
        .as_os_str()
        .to_str()
        .ok_or(anyhow!("Cannot convert path to string"))?
        .to_string())
}

/// Get an environment variable and returns None whether not set
pub fn get_env_variable(var_name: &str) -> Option<String> {
    match env::var(var_name) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

/// Remove a suffix from a string, or return the original
pub fn remove_suffix<'a>(in_string: &'a str, suffix: &str) -> &'a str {
    if let Some(result) = in_string.strip_suffix(suffix) {
        result
    } else {
        in_string
    }
}

pub fn relative_home_path_str(val: &str) -> Result<String> {
    let path = relative_home_path(val)?;
    string_from_path(&path)
}

pub fn relative_home_path(val: &str) -> Result<PathBuf> {
    let mut home_path = home::home_dir().ok_or(anyhow!("Can't get your home directory"))?;
    home_path.push(val);
    Ok(home_path)
}
