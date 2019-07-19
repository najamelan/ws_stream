# ws_stream examples

1. **echo** is a simple echo server that copies bytes from the AsyncRead to the AsyncWrite. It shows how to use BufReader
   to improve performance. It is used for the integration tests of the [ws_stream_wasm crate](https://crates.io/crates/ws_stream_wasm).
2. **echo_tt** is an echo server that just uses tokio-tungstenite, and not ws_stream. It can be used to measure the performance
   loss from adding ws_stream on top of tokio-tungstenite. It is also used for the integration tests of the [ws_stream_wasm crate](https://crates.io/crates/ws_stream_wasm), since ws_stream does not support WebSocket Text messages.
3. **tokio_codec** shows how you can frame a websocket connection with a tokio codec.
4. **futures_codec** TODO: shows how you can frame a websocket with futures-codec.
5. **chat_server** implements a simple chat server with ws_stream. [ws_stream_wasm](https://crates.io/crates/ws_stream_wasm)
   the [chat_client](https://github.com/najamelan/ws_stream_wasm/tree/master/examples/chat_client) that runs in WASM.
6. **chat_client** TODO: create a chat client with a terminal interface (if I'm really motivated ;)
