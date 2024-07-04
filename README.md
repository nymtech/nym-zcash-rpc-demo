# Nym-Zcash-RPC 

Interacting with a `zcashd` node via RPC over the mixnet using `zcash-cli`.

## Overview 
* `src/client` transports bytes between `zcash-cli` and the Nym 'server' instance via the mixnet.
* `src/server` interacts with a `zcashd` testnet fullnode and sends responses back to the 'client' using anonymous replies (**S**ingle **U**se **R**eply **B**locks).

```
+-----------+                                                                                                     
| zcash-cli |                                                                                                     
+-----------+                                                                                                     
      ^                                                                                                         
      |                                                                                                           
      v                                                                                                           
+----------------------+           +--------------------------------------------+         +----------------------+
| Nym client-side code |<--------->| Mixnet: Gateway -> Mixnode 1..3 -> Gateway |<------->| Nym server-side code |
+----------------------+           +--------------------------------------------+         +----------------------+
                                                                                                     ^            
                                                                                                     |            
                                                                                                     |            
                                                                                                     |            
                                                                                          +----------v----------+ 
                                                                                          | Zcashd testnet node | 
                                                                                          +---------------------+ 
```

## Binary Usage
```
# Server 
Usage: server [OPTIONS] --upstream-address <UPSTREAM_ADDRESS>

Options:
  -u, --upstream-address <UPSTREAM_ADDRESS>  Upstream address, ie lightwalletd address
  -c, --config-dir <CONFIG_DIR>              Config directory [default: /tmp/mixnet-client]
  -h, --help                                 Print help

# Client

Usage: client [OPTIONS] --server-address <SERVER_ADDRESS>

Options:
      --send-timeout <SEND_TIMEOUT>        Send timeout in seconds [default: 1]
      --receive-timeout <RECEIVE_TIMEOUT>  Receive timeout in seconds [default: 3]
  -r, --retry-count <RETRY_COUNT>          Send and receive retry count [default: 3]
  -s, --server-address <SERVER_ADDRESS>    Mixnet server address
      --listen-address <LISTEN_ADDRESS>    Listen address [default: 127.0.0.1]
      --listen-port <LISTEN_PORT>          Listen port [default: 8080]
  -h, --help                               Print help
```

## Run
```bash
# have a `zcashd` node running in the background: this has been tested using the zcash testnet 
# fund address using https://faucet.zecpages.com/

# terminal window 1:
cargo run --bin server -- -u <UPSTREAM_ADDRESS> # the local port that the zcash node is listening on e.g. "127.0.0.1:18232"

# terminal window 2: 
cargo run --bin client -- -s <SERVER_NYM_ADDR> # this will start listening on localhost:8080 by default

# specify the port that the client is listening on to pipe `zcash-cli` traffic through the mixnet, otherwise use normally 
zcash-cli -testnet -rpcport=8080 <COMMAND> 

# you will see traffic being sent between the two nym 'endpoints'
```

## Demo 
For ease of demonstration purposes, there is a `send_tx.sh` script which sends a tx through the mixnet and then queries the tx, returning the tx information. This script relies on `zcash-cli` being in your `$PATH`. 

