use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};

use serde::{de::DeserializeOwned, Serialize};

pub mod broadcast;
pub mod discovery;

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
}

pub trait Network<T> {
    fn await_events(&mut self);

    fn recv(&mut self) -> Option<T>;

    fn send(&mut self, msg: &T);
}

/// An in-memory testing network to help with unit testing
pub struct TestNetwork<T> {
    rx: Receiver<T>,
    tx: Sender<T>,
    buffer: Option<T>,
}

impl<T> TestNetwork<T> {
    pub fn new() -> (Self, Self) {
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        let network1 = Self {
            rx: rx1,
            tx: tx2,
            buffer: None,
        };
        let network2 = Self {
            rx: rx2,
            tx: tx1,
            buffer: None,
        };
        (network1, network2)
    }
}

impl<T: Send + Clone> Network<T> for TestNetwork<T> {
    fn await_events(&mut self) {
        if self.buffer.is_none() {
            self.buffer = self.rx.recv().ok();
        }
    }

    fn recv(&mut self) -> Option<T> {
        match self.buffer.take() {
            Some(msg) => Some(msg),
            None => self.rx.try_recv().ok(),
        }
    }

    fn send(&mut self, msg: &T) {
        self.tx.send(msg.clone()).unwrap();
    }
}
