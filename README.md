# BlockChat

Semester long project for the Distributed Systems class at NTUA.

## Assignment and final report

The description of the assignment can be found in
[DistributedProject2024.pdf](https://github.com/petrosagg/blockchat/blob/main/DistributedProject2024.pdf)
and the final report in
[BlockChat.pdf](https://github.com/petrosagg/blockchat/blob/main/BlockChat.pdf)

## Running

In order to run this project a working Rust installation is required. For
information about how to install Rust on your computer please visit
https://rustup.rs/.

In order to run a test cluster of 3 nodes on a single machine you need to run
one leader node like so:

```
cargo run --bin node -- --peers 3 --bootstrap-leader
```

And run two more nodes on two different terminals using the following command.
Notice that this time you don't have to pass the `--bootstrap-leader` argument.

```
cargo run --bin node -- --peers 3
```

After the blockchain is up and running you will have each node listening for
CLI instances on port `10000 + node_id`. For the three node example that would
be ports 10000, 10001, and 10002.

In order to connect to one of those nodes use the following command:

```
cargo run --bin cli -- --rpc-url='http://127.0.0.1:10001'
```

You can use the CLI command `help` to show the available CLI commands.

## Tests

The code includes unit tests that can be ran with `cargo test`.
