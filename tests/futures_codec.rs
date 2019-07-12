#![ feature( async_await ) ]

// Test using the AsyncRead/AsyncWrite from futures 0.3
//
// ✔ frame with futures-codec
// ✔ send/receive half a frame
//


use
{
	ws_stream     :: { *                                    } ,
	async_runtime :: { rt, RtConfig                         } ,
	futures       :: { StreamExt, SinkExt, channel::oneshot } ,
	futures_codec :: { LinesCodec, Framed                   } ,
	// log           :: { * } ,
};


#[ test ]
//
fn frame03()
{
	// flexi_logger::Logger::with_str( "events=trace, wasm_websocket_stream=trace, tokio=warn" ).start().expect( "flexi_logger");

	rt::init( RtConfig::Local ).expect( "rt::init" );


	let server = async
	{
		let mut connections = WsStream::listen( "127.0.0.1:3012" ).take(1);
		let     server      = connections.next().await.expect( "1 connection" ).expect( "1 connection" ).await.expect( "WS handshake" );

		let mut framed = Framed::new( server, LinesCodec {} );

		framed.send( "A line\n"       .to_string() ).await.expect( "Send a line" );
		framed.send( "A second line\n".to_string() ).await.expect( "Send a line" );
	};


	let client = async
	{
		let     client = WsStream::connect( "127.0.0.1:3012" ).await.expect( "connect to websocket" );
		let mut framed = Framed::new( client, LinesCodec {} );


		let res = framed.next().await.expect( "Receive some" ).expect( "Receive a line" );
		assert_eq!( "A line\n".to_string(), res );


		let res = framed.next().await.expect( "Receive some" ).expect( "Receive a second line" );
		assert_eq!( "A second line\n".to_string(), res );


		let res = framed.next().await;
		assert!( res.is_none() );
	};

	rt::spawn( server ).expect( "spawn task" );
	rt::spawn( client ).expect( "spawn task" );

	rt::run();
}


// Receive half a frame
//
#[ test ]
//
fn partial()
{
	// flexi_logger::Logger::with_str( "events=trace, wasm_websocket_stream=trace, tokio=warn" ).start().expect( "flexi_logger");

	rt::init( RtConfig::Local ).expect( "rt::init" );

	let (tx, rx) = oneshot::channel();

	let server = async move
	{
		let mut connections = WsStream::listen( "127.0.0.1:3013" ).take(1);
		let     server      = connections.next().await.expect( "1 connection" ).expect( "1 connection" ).await.expect( "WS handshake" );

		let mut framed = Framed::new( server, LinesCodec {} );

		framed.send( "A "             .to_string() ).await.expect( "Send a line" );

		// Make sure the client tries to read on a partial line first.
		//
		rx.await.expect( "read channel" );

		framed.send( "line\n"         .to_string() ).await.expect( "Send a line" );
		framed.send( "A second line\n".to_string() ).await.expect( "Send a line" );
	};

	let client = async move
	{
		let     client = WsStream::connect( "127.0.0.1:3013" ).await.expect( "connect to websocket" );
		let mut framed = Framed::new( client, LinesCodec {} );

		tx.send(()).expect( "trigger channel" );
		let res = framed.next().await.expect( "Receive some" ).expect( "Receive a line" );
		assert_eq!( "A line\n".to_string(), res );


		let res = framed.next().await.expect( "Receive some" ).expect( "Receive a second line" );
		assert_eq!( "A second line\n".to_string(), res );


		let res = framed.next().await;
		assert!( res.is_none() );
	};

	rt::spawn( server ).expect( "spawn task" );
	rt::spawn( client ).expect( "spawn task" );

	rt::run();
}
