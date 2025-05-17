# Secure UDP Communication Library

A secure, no_std compatible UDP communication library with encryption support, designed for embedded systems and resource-constrained environments.

## Project Overview

This project implements a secure UDP-based communication protocol with the following features:
- No standard library requirement (no_std compatible)
- Secure session establishment using Noise Protocol Framework
- Encrypted message exchange
- Reliable message delivery with sequencing and acknowledgment
- Lightweight implementation suitable for embedded systems

## Project Structure

The project is organized into three main crates:

### 1. `shared_lib`
The core library containing shared data structures and protocols:
- `command.rs`: Defines the command structures and message types for communication
- `network.rs`: Implements the network protocol with message headers and serialization
- `serialize.rs`: Provides serialization/deserialization utilities
- `error.rs`: Defines error types used across the codebase

### 2. `server`
The server implementation that handles incoming UDP connections:
- Implements session management
- Processes encrypted messages
- Handles handshake and message acknowledgment
- Maintains connection state

### 3. `client`
The client implementation that communicates with the server:
- Handles secure session establishment
- Manages message encryption/decryption
- Implements message sequencing and acknowledgment
- Provides a simple API for sending data

## Features

- **No Standard Library**: Designed to work in `no_std` environments
- **Secure Communication**: Implements Noise Protocol Framework for encryption
- **Lightweight**: Minimal dependencies and efficient memory usage
- **Reliable**: Includes message sequencing and acknowledgment
- **Simple API**: Easy-to-use interface for sending and receiving messages

## Building the Project

The project uses Cargo for building. Make sure you have Rust installed on your system.

### Prerequisites

- Rust toolchain (latest stable version recommended)
- Cargo (comes with Rust)

### Build Commands

```bash
# Format the code
make format

# Run linter
make lint

# Build the project
cargo build

# Build in release mode
cargo build --release
```

## Running the Project

The project includes both client and server executables.

### Starting the Server

```bash
# Run the server
make run-server

# Or directly with cargo
cargo run --bin server
```

The server will start on `127.0.0.1:8080` by default.

### Running the Client

```bash
# Run the client
make run-client

# Or directly with cargo
cargo run --bin client
```

## Protocol Details

The communication protocol includes:

1. **Message Types**:
   - HandshakeRequest/Response for session establishment
   - EncryptedMessage for data transfer
   - ACK for message acknowledgment
   - Timeout for session expiration

2. **Message Header** (14 bytes):
   - Protocol ID (2 bytes)
   - Protocol version (1 byte)
   - Message Type (1 byte)
   - Device ID (4 bytes)
   - Session ID (2 bytes)
   - Sequence number (2 bytes)
   - Acknowledgment number (2 bytes)

3. **Encryption**:
   - Uses Noise Protocol Framework
   - ChaCha20-Poly1305 for encryption
   - BLAKE2s for hashing

## Example Usage

```rust
// Create a new session
let mut session = Session::new(device_id);

// Create a temperature message
let message = session.temperature_message()?;

// Send the message to the server
// (Implementation depends on your network stack)
```

## License

MIT License