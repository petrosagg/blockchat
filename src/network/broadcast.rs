//! Implementation of a broadcasting network

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::network::Network;

pub struct Broadcaster<T> {
    write_txs: Vec<Sender<T>>,
    read_rx: Receiver<T>,
    buffer: Option<T>,
}

impl<T: Serialize + DeserializeOwned + Clone + Send + 'static> Broadcaster<T> {
    pub fn new(peers: &[SocketAddr], my_index: usize) -> Self {
        let sockets = create_sockets(peers, my_index);

        let (read_tx, read_rx) = mpsc::channel();
        let mut write_txs = vec![];
        for mut socket in sockets {
            let mut read_socket = BufReader::new(socket.try_clone().unwrap());
            let read_tx = read_tx.clone();
            std::thread::spawn(move || {
                let mut buf = String::new();
                loop {
                    match read_socket.read_line(&mut buf) {
                        Ok(0) => {
                            println!("Peer EOF");
                            return;
                        }
                        Ok(_) => {
                            read_tx.send(serde_json::from_str(&buf).unwrap()).unwrap();
                        }
                        Err(err) => {
                            println!("Connection error: {err}");
                            return;
                        }
                    }
                }
            });

            let (write_tx, write_rx) = mpsc::channel();
            std::thread::spawn(move || {
                while let Ok(msg) = write_rx.recv() {
                    serde_json::to_writer(&mut socket, &msg).unwrap();
                    socket.write(&[b'\n']).unwrap();
                    socket.flush().unwrap();
                }
            });
            write_txs.push(write_tx);
        }
        Self {
            write_txs,
            read_rx,
            buffer: None,
        }
    }
}

impl<T: Serialize + DeserializeOwned + Clone + Send + 'static> Network<T> for Broadcaster<T> {
    fn await_events(&mut self) {
        if self.buffer.is_none() {
            self.buffer = self.read_rx.recv().ok();
        }
    }

    fn recv(&mut self) -> Option<T> {
        match self.buffer.take() {
            Some(msg) => Some(msg),
            None => self.read_rx.try_recv().ok(),
        }
    }

    fn send(&mut self, msg: &T) {
        for write_tx in self.write_txs.iter_mut() {
            write_tx.send(msg.clone()).unwrap();
        }
    }
}

/// Creates a TCP stream for all peers.
fn create_sockets(peers: &[SocketAddr], my_index: usize) -> Vec<TcpStream> {
    std::thread::scope(|s| {
        let start_task = s.spawn(|| start_connections(&peers[..my_index]));
        let await_task = s.spawn(|| await_connections(peers[my_index], peers.len() - my_index - 1));

        let mut streams = start_task.join().unwrap();
        streams.extend(await_task.join().unwrap());
        streams
    })
}

/// Connects to the provided list of peers. Returns the established TCP streams.
fn start_connections(peers: &[SocketAddr]) -> Vec<TcpStream> {
    let mut streams = vec![];
    println!("Connecting to {} peers", peers.len());
    for peer in peers {
        // Make 5 attempts at connecting
        // TODO(petrosagg): Replace with the retry crate
        for attempt in 1..=5 {
            println!("Connecting to {peer} attempt {attempt}");

            match TcpStream::connect(peer) {
                Ok(stream) => {
                    println!("Successful connection to {peer}");
                    stream.set_nodelay(true).expect("set_nodelay call failed");
                    streams.push(stream);
                    break;
                }
                Err(error) => {
                    println!("Failed connecting to {peer}: {error}");
                    std::thread::sleep(Duration::from_millis(200));
                }
            }
        }
    }
    streams
}

/// Waits for the expected number of peers to connect. Returns the established TCP streams.
fn await_connections(listen_addr: SocketAddr, expected_peers: usize) -> Vec<TcpStream> {
    let mut streams = vec![];
    let listener = TcpListener::bind(listen_addr).unwrap();

    for _ in 0..expected_peers {
        let stream = listener.accept().unwrap().0;
        stream.set_nodelay(true).expect("set_nodelay call failed");
        streams.push(stream);
    }
    streams
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_networking() {
        let addrs = [
            "127.0.0.1:6000".parse().unwrap(),
            "127.0.0.1:6001".parse().unwrap(),
            "127.0.0.1:6002".parse().unwrap(),
        ];
        std::thread::scope(|s| {
            s.spawn(|| {
                let mut peer = Broadcaster::<usize>::new(&addrs, 0);
                peer.await_events();
                assert_eq!(peer.recv(), Some(42));
            });
            s.spawn(|| {
                let mut peer = Broadcaster::<usize>::new(&addrs, 1);
                peer.send(&42);
            });
            s.spawn(|| {
                let mut peer = Broadcaster::<usize>::new(&addrs, 2);
                peer.await_events();
                assert_eq!(peer.recv(), Some(42));
            });
        })
    }
}
