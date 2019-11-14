# ws_stream

This crate has been split in several subcrates:

- [ws_stream_wasm](https://crates.io/crates/ws_stream_wasm): Creates an idiomatic Rust interface to the browser websocket API (in WASM) and provides AsyncRead/AsyncWrite enabling communication using framing with a codec and treating the connection as a generic TCP stream of bytes.
- [ws_stream_tungstenite](https://crates.io/crates/ws_stream_tungstenite): A layer on top of [tokio-tungstenite](https://crates.io/crates/tokio-tungstenite) that implements AsyncRead/AsyncWrite, enabling communication using framing with a codec and treating the connection as a generic TCP stream of bytes.

The purpose is to allow network libraries that can work on any AsyncRead/AsyncWrite to function over websockets.
