use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

use serde::{de::DeserializeOwned, Serialize};

use crate::network::TypedJsonStream;

/// Connects to the specified bootstrap server and returns a list of addreses for all the nodes in
/// the network.
pub fn discover_peers<D>(bootstrap_addr: SocketAddr, data: D) -> (usize, Vec<D>)
where
    D: Serialize + DeserializeOwned,
{
    let socket = loop {
        match TcpStream::connect(bootstrap_addr) {
            Ok(stream) => break stream,
            // TODO(petrosagg): replace with retry crate
            Err(_) => std::thread::sleep(Duration::from_millis(200)),
        }
    };
    let mut stream = TypedJsonStream::new(socket);

    stream.send(&data);
    (stream.recv(), stream.recv())
}

pub fn bootstrap_helper<D>(bootstrap_addr: SocketAddr, expected_peers: usize)
where
    D: Serialize + DeserializeOwned,
{
    let listener = TcpListener::bind(bootstrap_addr).unwrap();

    let mut streams = vec![];
    let mut peer_data = vec![];
    for _ in 0..expected_peers {
        let socket = listener.accept().unwrap().0;
        let mut stream = TypedJsonStream::new(socket);
        let data = stream.recv::<D>();
        let index = streams.len();
        streams.push((index, stream));
        peer_data.push(data);
    }

    for (peer_index, mut peer_stream) in streams {
        peer_stream.send(&peer_index);
        peer_stream.send(&peer_data);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_discovery() {
        let bootstrap_addr = "127.0.0.1:7001".parse().unwrap();
        std::thread::scope(|s| {
            // First spawn the bootstrap helper
            s.spawn(|| bootstrap_helper::<(SocketAddr, u64)>(bootstrap_addr, 3));

            // Then each peer performs discovery
            s.spawn(|| {
                let addr: SocketAddr = "127.0.0.1:6000".parse().unwrap();
                let (my_index, data) = discover_peers(bootstrap_addr, (addr, 1));
                assert_eq!(data[my_index], (addr, 1));
                assert_eq!(data.len(), 3);
            });
            s.spawn(|| {
                let addr: SocketAddr = "127.0.0.1:6001".parse().unwrap();
                let (my_index, data) = discover_peers(bootstrap_addr, (addr, 2));
                assert_eq!(data[my_index], (addr, 2));
                assert_eq!(data.len(), 3);
            });
            s.spawn(|| {
                let addr: SocketAddr = "127.0.0.1:6002".parse().unwrap();
                let (my_index, data) = discover_peers(bootstrap_addr, (addr, 3));
                assert_eq!(data[my_index], (addr, 3));
                assert_eq!(data.len(), 3);
            });
        })
    }
}
