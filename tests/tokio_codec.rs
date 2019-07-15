#![ feature( async_await ) ]
#![ cfg(not( target_arch = "wasm32" )) ]

// To test:
//
// ✔ frame with tokio codec
// ✔ verify task wakes up after WouldBlock


use
{
	ws_stream     :: { *                                              } ,
	async_runtime :: { rt, RtConfig                                   } ,
	futures       :: { StreamExt, SinkExt, channel::oneshot           } ,
	futures       :: { compat::{ Stream01CompatExt, Sink01CompatExt } } ,
	futures_01    :: { stream::Stream as _                            } ,
	tokio         :: { codec::{ LinesCodec, Decoder }                 } ,
	// log           :: { * } ,
};


#[ test ]
//
fn frame()
{
	// flexi_logger::Logger::with_str( "events=trace, wasm_websocket_stream=trace, tokio=warn" ).start().expect( "flexi_logger");

	rt::init( RtConfig::Local ).expect( "rt::init" );


	let server = async
	{
		let mut connections = WsStream::listen( "127.0.0.1:3112" ).take(1);
		let     server      = connections.next().await.expect( "1 connection" ).expect( "1 connection" ).await.expect( "WS handshake" );

		let codec = LinesCodec::new();

		let (sink, _stream) = codec.framed( server ).split();
		let mut sink       = sink.sink_compat();

		sink.send( "A line"       .to_string() ).await.expect( "Send a line" );
		sink.send( "A second line".to_string() ).await.expect( "Send a line" );
	};


	let client = async
	{
		let client = WsStream::connect( "127.0.0.1:3112" ).await.expect( "connect to websocket" );

		let codec = LinesCodec::new();

		let (_sink, stream) = codec.framed( client ).split();
		let mut stream      = stream.compat();

		let res = stream.next().await.expect( "Receive a line" ).expect( "Receive a line" );

		assert_eq!( "A line".to_string(), res );

		let res = stream.next().await.expect( "Receive a line" ).expect( "Receive a line" );

		assert_eq!( "A second line".to_string(), res );
	};


	rt::spawn( server ).expect( "spawn task" );
	rt::spawn( client ).expect( "spawn task" );

	rt::run();
}



// This test obliges the sender taks to wait before sending again, asuring a WouldBlock happens on the
// reader. We then make sure the task get's woken up to read the second write.
//
#[ test ]
//
fn would_block()
{
	// flexi_logger::Logger::with_str( "events=trace, ws_stream=trace, wasm_websocket_stream=trace, tokio=warn" ).start().expect( "flexi_logger");

	rt::init( RtConfig::Local ).expect( "rt::init" );

	let (tx, rx) = oneshot::channel();


	let server = async move
	{
		let mut connections = WsStream::listen( "127.0.0.1:3113" ).take(1);
		let     server      = connections.next().await.expect( "1 connection" ).expect( "1 connection" ).await.expect( "WS handshake" );

		let codec = LinesCodec::new();

		let (sink, _stream) = codec.framed( server ).split();
		let mut sink        = sink.sink_compat();

		sink.send( "A line".to_string() ).await.expect( "Send a line" );

		rx.await.expect( "await channel" );

		sink.send( "A second line".to_string() ).await.expect( "Send a line" );
	};


	let client = async move
	{
		let client = WsStream::connect( "127.0.0.1:3113" ).await.expect( "connect to websocket" );

		let codec = LinesCodec::new();

		let (sink, stream) = codec.framed( client ).split();
		let _sink          = sink.sink_compat();
		let mut stream     = stream.compat();

		let res = stream.next().await.expect( "Receive some" ).expect( "Receive a line" );
		assert_eq!( "A line".to_string(), res );

		tx.send(()).expect( "Trigger channel" );
		let res = stream.next().await;

		let line = res.expect( "Receive some" ).expect( "Receive a line" );

		assert_eq!( "A second line".to_string(), line );
	};


	rt::spawn( server ).expect( "spawn task" );
	rt::spawn( client ).expect( "spawn task" );

	rt::run();
}
