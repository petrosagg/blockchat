use std::net::{SocketAddr, TcpStream, TcpListener};

use serde::{de::DeserializeOwned, Serialize};

use crate::network::TypedJsonStream;


/// Connects to the specified bootstrap server and returns a list of addreses for all the nodes in
/// the network.
fn discover_peers<D: Serialize + DeserializeOwned>(bootstrap_addr: SocketAddr, my_addr: SocketAddr, data: D) -> (usize, Vec<SocketAddr>, Vec<D>) {
    let socket = TcpStream::connect(bootstrap_addr).unwrap();
    let mut stream = TypedJsonStream::new(socket);

    stream.send(&my_addr);
    stream.send(&data);
    let my_index: usize = stream.recv();
    let peer_addrs: Vec<SocketAddr> = stream.recv();
    let peer_data: Vec<String> = stream.recv();
    let peer_data = peer_data.into_iter().map(|data| serde_json::from_str(&data).unwrap()).collect();
    (my_index, peer_addrs, peer_data)
}

fn bootstrap_helper(bootstrap_addr: SocketAddr, expected_peers: usize) {
    let listener = TcpListener::bind(bootstrap_addr).unwrap();

    let mut streams = vec![];
    let mut peer_addrs = vec![];
    let mut peer_data = vec![];
    for _ in 0..expected_peers {
        let mut socket = listener.accept().unwrap().0;
        let mut stream = TypedJsonStream::new(socket);
        let addr = stream.recv::<SocketAddr>();
        let data = stream.recv_raw().to_owned();
        let index = streams.len();
        streams.push((index, stream));
        peer_addrs.push(addr);
        peer_data.push(data);
    }

    for (peer_index, mut peer_stream) in streams {
        peer_stream.send(&peer_index);
        peer_stream.send(&peer_addrs);
        peer_stream.send(&peer_data);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_discovery() {
        let bootstrap_addr = "127.0.0.1:7000".parse().unwrap();
        let addr1 = "127.0.0.1:6000".parse().unwrap();
        let addr2 = "127.0.0.1:6001".parse().unwrap();
        let addr3 = "127.0.0.1:6002".parse().unwrap();
        std::thread::scope(|s| {
            // First spawn the bootstrap helper
            s.spawn(|| bootstrap_helper(bootstrap_addr, 3));

            // Then each peer performs discovery
            s.spawn(|| {
                let (my_index, addrs, data) = discover_peers(bootstrap_addr, addr1, 1);
                assert_eq!(addrs[my_index], addr1);
                assert_eq!(addrs.len(), 3);
                assert_eq!(data[my_index], 1);
                assert_eq!(data.len(), 3);
            });
            s.spawn(|| {
                let (my_index, addrs, data) = discover_peers(bootstrap_addr, addr2, 2);
                assert_eq!(addrs[my_index], addr2);
                assert_eq!(addrs.len(), 3);
                assert_eq!(data[my_index], 2);
                assert_eq!(data.len(), 3);
            });
            s.spawn(|| {
                let (my_index, addrs, data) = discover_peers(bootstrap_addr, addr3, 3);
                assert_eq!(addrs[my_index], addr3);
                assert_eq!(addrs.len(), 3);
                assert_eq!(data[my_index], 3);
                assert_eq!(data.len(), 3);
            });
        })
    }
}
