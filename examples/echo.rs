//! This is an echo server that returns all incoming bytes, without framing. It is used for the tests in
//! ws_stream_wasm.
//
use
{
	ws_stream     :: { *                                                       } ,
	async_runtime :: { rt, RtConfig                                            } ,
	futures       :: { StreamExt, AsyncReadExt, AsyncBufReadExt, io::BufReader } ,
	std           :: { env                                                     } ,
	log           :: { *                                                       } ,
};



fn main()
{
	// flexi_logger::Logger::with_str( "echo=trace, ws_stream=trace, tokio=trace, tokio_tungstenite=trace, futures_util=trace, futures=trace, futures_io=trace" ).start().unwrap();

	// We only need one thread.
	//
	rt::init( RtConfig::Local ).expect( "init rt" );


	let server = async move
	{
		let addr: String = env::args().nth(1).unwrap_or( "127.0.0.1:3212".to_string() ).parse().unwrap();
		println!( "server task listening at: {}", &addr );


		let mut connections = TungWebSocket::listen( &addr );

		while let Some( stream ) = connections.next().await
		{
			let conn = async move
			{
				// If the TCP stream fails, we stop processing this connection
				//
				let tcp_stream = match stream
				{
					Ok(tcp) => tcp,
					Err(_) =>
					{
						debug!( "Failed TCP incoming connection" );
						return;
					}
				};

				// If the Ws handshake fails, we stop processing this connection
				//
				let socket = match tcp_stream.await
				{
					Ok(ws) => ws,

					Err(_) =>
					{
						debug!( "Failed WebSocket HandShake" );
						return;
					}
				};


				info!( "Incoming connection from: {}", socket.peer_addr().expect( "peer addr" ) );

				let ws_stream = WsStream::new( socket );
				let (reader, mut writer) = ws_stream.split();

				// BufReader allows our AsyncRead to work with a bigger buffer than the default 8k.
				// This improves performance quite a bit.
				//
				match BufReader::with_capacity( 64_000, reader ).copy_buf_into( &mut writer ).await
				{
					Ok (_) => {},

					Err(e) =>
					{
						error!( "{}", e );
					}
				}
			};

			rt::spawn( conn ).expect( "spawn conn" );
		}
	};

	rt::spawn( server ).expect( "spawn task" );
	rt::run();
}

