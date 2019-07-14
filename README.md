# ws_stream


## TODO

- write echo server (used for tests) using ws_stream and maybe put it in examples
- unit tests working just on AsyncRead/AsyncWrite, but that don't frame.
- verify gloo discussion on design and API
- all `TODO` and `FIXME` in `src/` and `test/`
- examples
- reread everything
- documentation

In order to run the tests, go into the server crate and run `cargo run`. In another tab run `wasm-pack test  --firefox --headless`.

little note on performance:
  - `wasm-pack test  --firefox --headless --release`: 13.3s
  - `wasm-pack test  --chrome  --headless --release`: 10.4s


  - callback_future is not something that belongs in wasm_websocket_stream. Maybe a functionality like that can be added to wasm_bindgen? It should also be possible to do this for callbacks that need to take parameters.

  - the sink? It's always ready to write, it's always flushed? The websocket browser api does not really give any info about the state here... Need better unit testing to verify what happens under stress I suppose.

  - We don't have Clone or Eq on JsWebSocket or WsStream... Is that ok?

