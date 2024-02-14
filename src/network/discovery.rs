//! Peer discovery through a designated bootstrap node.

use std::net::{SocketAddr, TcpStream, TcpListener};

use crate::network::TypedJsonStream;


/// Connects to the specified bootstrap server and returns a list of addreses for all the nodes in
/// the network.
fn discover_peers(bootstrap_addr: SocketAddr, my_addr: SocketAddr) -> (usize, Vec<SocketAddr>) {
    let socket = TcpStream::connect(bootstrap_addr).unwrap();
    let mut stream = TypedJsonStream::new(socket);

    stream.send(&my_addr);
    stream.recv()
}

fn bootstrap_helper(bootstrap_addr: SocketAddr, expected_peers: usize) {
    let listener = TcpListener::bind(bootstrap_addr).unwrap();

    let mut streams = vec![];
    let mut addrs = vec![];
    for _ in 0..expected_peers {
        let mut socket = listener.accept().unwrap().0;
        let mut peer_stream = TypedJsonStream::new(socket);
        let peer_addr = peer_stream.recv::<SocketAddr>();
        let peer_index = streams.len();
        streams.push((peer_index, peer_stream));
        addrs.push(peer_addr);
    }

    for (peer_index, mut peer_stream) in streams {
        peer_stream.send(&(peer_index, addrs.clone()));
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
                let (my_index, peers) = discover_peers(bootstrap_addr, addr1);
                assert_eq!(peers[my_index], addr1);
                assert_eq!(peers.len(), 3);
            });
            s.spawn(|| {
                let (my_index, peers) = discover_peers(bootstrap_addr, addr2);
                assert_eq!(peers[my_index], addr2);
                assert_eq!(peers.len(), 3);
            });
            s.spawn(|| {
                let (my_index, peers) = discover_peers(bootstrap_addr, addr3);
                assert_eq!(peers[my_index], addr3);
                assert_eq!(peers.len(), 3);
            });
        })
    }
}
