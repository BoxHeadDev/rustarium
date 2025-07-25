# Rustarium

A collection of small rust projects

## Ideas

### Chat Server
An exercise in networking and asynchronous programming. 
Multiple client programs connect to a server program. 
A client can send a message either to a specific different client, or to all other clients (broadcast). 
There are many variations on how to implement this: 
- blocking read/write calls
- epoll
- io_uring
- threads
- callbacks
- futures
- manually-coded state machines

### Servers
- UDP (file transfer)
- TCP (chat)
- gRPC
- HTTP
- TCP
- GraphQL

### Protocols
- RESP (+ Tcp)
- Protocol Buffers (+ gRPC)
- JSON (+ HTTP)
- MessagePack
- CapNProto
- Custom

### Async 
- Tokio
- Async Std
- Futures 
