use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use serde::{de::DeserializeOwned, Serialize};

use crate::network::TypedJsonStream;

/// Connects to the specified bootstrap server and returns a list of addreses for all the nodes in
/// the network.
pub fn discover_peers<D1, D2>(bootstrap_addr: SocketAddr, data: D1) -> (usize, Vec<D1>, D2)
where
    D1: Serialize + DeserializeOwned,
    D2: Serialize + DeserializeOwned,
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
    (stream.recv(), stream.recv(), stream.recv())
}

pub fn bootstrap_helper<D1, D2>(
    bootstrap_addr: SocketAddr,
    expected_peers: usize,
    bootstrap_data: D2,
) where
    D1: Serialize + DeserializeOwned,
    D2: Serialize + DeserializeOwned,
{
    let listener = TcpListener::bind(bootstrap_addr).unwrap();

    let mut streams = vec![];
    let mut peer_data = vec![];
    for _ in 0..expected_peers {
        let socket = listener.accept().unwrap().0;
        let mut stream = TypedJsonStream::new(socket);
        let data = stream.recv::<D1>();
        let index = streams.len();
        streams.push((index, stream));
        peer_data.push(data);
    }

    for (peer_index, mut peer_stream) in streams {
        peer_stream.send(&peer_index);
        peer_stream.send(&peer_data);
        peer_stream.send(&bootstrap_data);
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
            s.spawn(|| bootstrap_helper::<(SocketAddr, u64), u64>(bootstrap_addr, 3, 42));

            // Then each peer performs discovery
            s.spawn(|| {
                let addr: SocketAddr = "127.0.0.1:6000".parse().unwrap();
                let (my_index, peer_data, bootstrap_data) =
                    discover_peers::<_, u64>(bootstrap_addr, (addr, 1));
                assert_eq!(peer_data[my_index], (addr, 1));
                assert_eq!(peer_data.len(), 3);
                assert_eq!(bootstrap_data, 42);
            });
            s.spawn(|| {
                let addr: SocketAddr = "127.0.0.1:6001".parse().unwrap();
                let (my_index, peer_data, bootstrap_data) =
                    discover_peers::<_, u64>(bootstrap_addr, (addr, 2));
                assert_eq!(peer_data[my_index], (addr, 2));
                assert_eq!(peer_data.len(), 3);
                assert_eq!(bootstrap_data, 42);
            });
            s.spawn(|| {
                let addr: SocketAddr = "127.0.0.1:6002".parse().unwrap();
                let (my_index, peer_data, bootstrap_data) =
                    discover_peers::<_, u64>(bootstrap_addr, (addr, 3));
                assert_eq!(peer_data[my_index], (addr, 3));
                assert_eq!(peer_data.len(), 3);
                assert_eq!(bootstrap_data, 42);
            });
        })
    }
}
