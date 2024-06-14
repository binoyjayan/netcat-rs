# netcat-rs

🦀 🦀 🦀 A netcat implementation in Rust 🦀 🦀 🦀

## Build and Install

Build netcat and copy the binary to a directory that is in PATH.

```
cargo build --release
sudo cp target/release/netcat /usr/local/bin/
```

## Running netcat

### TCP server and client

Run TCP client and server in separate terminals

Server:
```
netcat -l 1234
```

Client:
```
netcat localhost 1234
```

Listen on and connect over a different IP

Server:
```
netcat -l 1234 192.168.0.1
```

Client:
```
netcat 192.168.0.1 1234
```

### UDP server and client

Run UDP client and server in separate terminals

Server:
```
netcat -u -l 1234
```

Client:
```
netcat -u localhost 1234
```

Listen on and connect over a different IP

Server:
```
netcat -u -l 1234 192.168.0.1
```

Client:
```
netcat -u 192.168.0.1 1234
```

## Communicate over TLS

Communicate over TLS with and without client verification

### Generate keys and certificates

```
./test/gen-cert.sh
```

### TLS without client verification

Server:
```
netcat -l 1234 -C .ca.pem -c .server.pem -k .server-key.pem
```

Client:
```
netcat -C .ca.pem localhost 1234
```

### TLS with client verification [mTLS]

Server:
```
netcat -l 1234 -C .ca.pem -c .server.pem -k .server-key.pem --client-auth
```

Client:
```
netcat -C .ca.pem -c .client.pem -k .client-key.pem  localhost 1234
```


### Running shell on server

Run commands on the client side and they will execute on the server.

Run shell on Server:
```
netcat -l 1234 -e /bin/bash
```

Connect to the shell:
```
netcat localhost 1234
```

### Reverse shell on client

Run commands on the server side and they will execute on the client.

Run Server:
```
netcat -l 1234
```

Connect to the shell:
```
netcat localhost 1234 -e /bin/bash
```

