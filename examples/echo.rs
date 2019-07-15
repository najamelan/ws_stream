#![ feature( async_await ) ]


// This is an echo server that returns all incoming bytes, without framing. It is used for the tests in
// ws_stream_wasm.


use
{
	ws_stream     :: { *                       } ,
	async_runtime :: { rt, RtConfig            } ,
	futures       :: { StreamExt, AsyncReadExt } ,
	std           :: { io                      } ,
};


fn main()
{
	// flexi_logger::Logger::with_str( "echo=trace, ws_stream=trace, tokio=warn" ).start().unwrap();

	rt::init( RtConfig::Local ).expect( "init rt" );


	let server = async
	{
		println!( "server task listening at: 127.0.0.1:3212"  );

		let mut connections = WsStream::listen( "127.0.0.1:3212" );

		while let Some( stream ) = connections.next().await
		{
			let conn = async move
			{
				let ws_stream = stream.expect( "tcp stream" ).await.expect( "ws handshake" );
				println!( "Incoming connection from: {}", ws_stream.peer_addr().expect( "peer addr" ) );

				let (reader, mut writer) = ws_stream.split();

				match reader.copy_into( &mut writer ).await
				{
					Ok(_) => {},
					Err(e) => match e.kind()
					{
						// This can happen in the flush, but it's becaue the client has already disconnected
						//
						io::ErrorKind::ConnectionAborted => {}

						// Other errors we want to know about
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

