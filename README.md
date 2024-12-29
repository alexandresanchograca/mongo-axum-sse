# MongoDB Real-time Updates via SSE with Axum

A proof-of-concept application demonstrating real-time updates from MongoDB to a web browser using Rust, Axum, and Server-Sent Events (SSE).

## Features

- Real-time MongoDB change stream monitoring
- Server-Sent Events (SSE) for push notifications
- Lightweight client-side JavaScript implementation
- Zero client-side dependencies

## Tech Stack

- **Backend**
  - Rust
  - [Axum](https://github.com/tokio-rs/axum) - Web framework
  - [MongoDB Rust Driver](https://github.com/mongodb/mongo-rust-driver)
  - [Tokio](https://tokio.rs/) - Async runtime
  
- **Frontend**
  - JavaScript
  - Server-Sent Events (SSE)
  - HTML5

## Prerequisites

- Rust (latest stable version)
- MongoDB 4.0 or higher (with replica set enabled)

## How It Works

1. The Rust backend establishes a change stream with MongoDB
2. Changes are broadcast to all connected clients using SSE
3. The client-side JavaScript renders updates in real-time

## Disclaimer

This is a proof-of-concept project and should not be used in production without proper security measures and error handling.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
