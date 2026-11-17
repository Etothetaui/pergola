//! Line-delimited JSON over a Unix socket.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub argv: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub ok: bool,
    pub message: String,
}

impl Response {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: message.into(),
        }
    }
}

/// Open a streaming subscription. The daemon takes the socket over and pushes
/// newline-delimited JSON records; the reader ends when the daemon exits.
pub fn subscribe(socket: &Path) -> Result<BufReader<UnixStream>> {
    let mut stream = UnixStream::connect(socket).with_context(|| {
        format!(
            "cannot reach the pergola daemon at {} (start it with `pergola daemon`)",
            socket.display()
        )
    })?;
    let request = Request {
        argv: vec!["subscribe".into()],
    };
    let mut line = serde_json::to_string(&request)?;
    line.push('\n');
    stream.write_all(line.as_bytes())?;
    stream.flush()?;
    Ok(BufReader::new(stream))
}

/// Send one request and read one response.
pub fn call(socket: &Path, argv: &[String]) -> Result<Response> {
    let mut stream = UnixStream::connect(socket).with_context(|| {
        format!(
            "cannot reach the pergola daemon at {} (start it with `pergola daemon`)",
            socket.display()
        )
    })?;
    let request = Request {
        argv: argv.to_vec(),
    };
    let mut line = serde_json::to_string(&request)?;
    line.push('\n');
    stream.write_all(line.as_bytes())?;
    stream.flush()?;
    let mut reader = BufReader::new(stream);
    let mut reply = String::new();
    reader.read_line(&mut reply)?;
    if reply.trim().is_empty() {
        anyhow::bail!("the daemon closed the connection without replying");
    }
    Ok(serde_json::from_str(reply.trim())?)
}
