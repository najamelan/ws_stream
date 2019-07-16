//! This is an echo server that returns all incoming bytes, without framing. It is used for the tests in
//! ws_stream_wasm.
//
#![ feature( async_await ) ]


use
{
	ws_stream     :: { *                       } ,
	async_runtime :: { rt, RtConfig            } ,
	futures       :: { StreamExt, AsyncReadExt, AsyncBufReadExt, io::BufReader } ,
	std           :: { io, env                      } ,
};



fn main()
{
	// flexi_logger::Logger::with_str( "echo=trace, ws_stream=trace, tokio=warn" ).start().unwrap();

	// We only need one thread.
	//
	rt::init( RtConfig::Local ).expect( "init rt" );


	let server = async move
	{
		let addr: String = env::args().nth(1).unwrap_or( "127.0.0.1:3212".to_string() ).parse().unwrap();
		println!( "server task listening at: {}", &addr );


		let mut connections = WsStream::listen( &addr );

		while let Some( stream ) = connections.next().await
		{
			let conn = async move
			{
				let ws_stream = stream.expect( "tcp stream" ).await.expect( "ws handshake" );
				println!( "Incoming connection from: {}", ws_stream.peer_addr().expect( "peer addr" ) );


				let (reader, mut writer) = ws_stream.split();

				// BufReader allows our AsyncRead to work with a bigger buffer than the default 8k.
				// This improves performance quite a bit.
				//
				match BufReader::with_capacity( 512_000, reader ).copy_buf_into( &mut writer ).await
				{
					Ok (_) => {},

					Err(e) => match e.kind()
					{
						// This can happen in the flush, but it's becaue the client has already disconnected
						//
						io::ErrorKind::NotConnected      => {}
						io::ErrorKind::ConnectionAborted => {}

						// If another error happens, we want to know about it
						//
						_ => { panic!( e ) }
					}
				}
			};

			rt::spawn( conn ).expect( "spawn conn" );
		}
	};

	rt::spawn( server ).expect( "spawn task" );
	rt::run();
}

