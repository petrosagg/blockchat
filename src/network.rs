use std::io::{BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use serde::{de::DeserializeOwned, Serialize};

pub mod broadcast;
pub mod discovery;

/// A wrapper over a TCP connection that is able to send and receive typed data
struct TypedStream {
    /// The underlying TCP stream.
    stream: BufReader<TcpStream>,
}

impl TypedStream {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufReader::new(stream),
        }
    }

    fn send<T: Serialize>(&mut self, msg: &T) {
        bincode::serialize_into(self.stream.get_mut(), &msg).unwrap();
        self.stream.get_mut().flush().unwrap();
    }

    fn recv<T: DeserializeOwned>(&mut self) -> T {
        bincode::deserialize_from(&mut self.stream).unwrap()
    }
}

pub trait Network<T> {
    fn await_events(&mut self, timeout: Option<Duration>);

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
    fn await_events(&mut self, timeout: Option<Duration>) {
        if self.buffer.is_none() {
            self.buffer = match timeout {
                Some(timeout) => self.rx.recv_timeout(timeout).ok(),
                None => self.rx.recv().ok(),
            };
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
