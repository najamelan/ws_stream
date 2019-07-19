//! An echo server using just tokio tungstenite. This allows comparing the
//! performance with ws_stream.
//
#![ feature( async_await ) ]


use
{
	async_runtime     :: { rt                                                                           } ,
	futures           :: { StreamExt, compat::{ Stream01CompatExt, Sink01CompatExt, Future01CompatExt } } ,
	futures_01        :: { future::{ ok, Future }, stream::Stream                                       } ,
	tokio_tungstenite :: { accept_async                                                                 } ,
	tokio             :: { net::TcpListener                                                             } ,
	std               :: { env                                                                          } ,
	log               :: { *                                                                            } ,
};


fn main()
{
	let program = async
	{
		flexi_logger::Logger::with_str( "echo_tt=trace, tokio=warn" ).start().unwrap();


		let addr         = env::args().nth(1).unwrap_or( "127.0.0.1:3212".to_string() ).parse().unwrap();
		let mut incoming = TcpListener::bind( &addr ).unwrap().incoming().compat();

		println!( "Listening on: {}", addr );


		while let Some( conn ) = incoming.next().await
		{
			// If the TCP stream fails, we stop processing this connection
			//
			let tcp_stream = match conn
			{
				Ok(tcp) => tcp,
				Err(_) =>
				{
					debug!( "Failed TCP incoming connection" );
					continue;
				}
			};


			let addr      = tcp_stream.peer_addr().expect( "connected streams should have a peer address" );
			let handshake = ok(tcp_stream).and_then( accept_async ).compat();


			// If the Ws handshake fails, we stop processing this connection
			//
			let ws_stream = match handshake.await
			{
				Ok(ws) => ws,

				Err(_) =>
				{
					debug!( "Failed WebSocket HandShake" );
					continue;
				}
			};

			let (sink, stream) = ws_stream.split();
			let stream         = stream.compat();
			let sink           = sink.sink_compat();

			println!( "New WebSocket connection: {}", addr );


			match stream.forward( sink ).await
			{
				Ok(()) => {},

				Err(e) => match e
				{
					// This can happen in the flush, but it's because the client has already disconnected
					// FIXME: probably our wasm code doesn't properly close the websocket
					//
					tungstenite::error::Error::ConnectionClosed => {}
					tungstenite::error::Error::AlreadyClosed    => {}

					// Other errors we want to know about
					//
					_ => { panic!( e ) }
				}
			}
		}
	};

	rt::block_on( program );
}
