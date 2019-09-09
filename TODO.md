# TODO

## Features

### Matrix

1. ✔ listen plain tokio-tungstenite
2. listen ssl   tokio-tungstenite
3. server with warp plain
4. server with warp https
5. connect plain tokio-tungstenite
6. connect ssl   tokio-tungstenite
7. connect wasm to 1-4 above

For each of the above:
- Good API (work with domain names and ip adresses)
- Error handling
- testing
- documentation/examples
- feature flag tungstenite, warp and ssl, make dependencies optional.

## Issues
- the connection closed error from tungstenite

## Testing
- need to close when using connect? wasm closes on drop
- https/wss, tls
- use tungwebsocket and warpwebsocket as sink stream without asyncread
- error handling
- spurious wakeups in poll_write
- enable travis CI
- enable dependabot


## Documentation
- chat client example
- documentation
