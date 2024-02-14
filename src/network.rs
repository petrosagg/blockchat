use std::{io::{BufReader, BufRead, Write}, net::TcpStream};

use serde::{de::DeserializeOwned, Serialize};

mod broadcast;
mod discovery;

/// A wrapper over a TCP connection that is able to send and receive data using line delimited
/// JSON.
struct TypedJsonStream {
    /// The underlying TCP stream.
    stream: BufReader<TcpStream>,
    /// A temporary buffer to hold the JSON data before decoding.
    buf: String,
}

impl TypedJsonStream {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufReader::new(stream),
            buf: String::new(),
        }
    }

    fn send<T: Serialize>(&mut self, msg: &T) {
        serde_json::to_writer(self.stream.get_mut(), &msg).unwrap();
        self.stream.get_mut().write(&[b'\n']).unwrap();
        self.stream.get_mut().flush().unwrap();
    }

    fn recv<T: DeserializeOwned>(&mut self) -> T {
        self.buf.clear();
        self.stream.read_line(&mut self.buf).unwrap();
        serde_json::from_str(&self.buf).unwrap()
    }

    fn send_raw(&mut self, msg: &str) {
        assert!(!msg.contains('\n'));
        self.stream.get_mut().write_all(msg.as_bytes());
        self.stream.get_mut().write_all(&[b'\n']).unwrap();
        self.stream.get_mut().flush().unwrap();
    }

    fn recv_raw(&mut self) -> &str {
        self.buf.clear();
        self.stream.read_line(&mut self.buf).unwrap();
        &self.buf
    }
}
