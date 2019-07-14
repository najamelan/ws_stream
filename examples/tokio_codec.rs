#![ feature( async_await ) ]

// We create a server and a client who connect over a websocket and frame that connection with
// a tokio codec.
//
fn main()
{
	use
	{
		ws_stream     :: { *                 } ,
		async_runtime :: { rt, RtConfig      } ,
		futures       :: { StreamExt, SinkExt } ,
		futures       :: { compat::{ Stream01CompatExt, Sink01CompatExt } } ,
		tokio         :: { codec::{ LinesCodec, Decoder } } ,
		log           :: { * } ,
	};

	flexi_logger::Logger::with_str( "events=trace, framed=trace, wasm_websocket_stream=trace, tokio=warn" ).start().unwrap();

	rt::init( RtConfig::Local ).expect( "init rt" );

	let server = async move
	{
		debug!( "server task" );

		let mut connections = WsStream::listen( "127.0.0.1:3112" ).take(1);
		let     server      = connections.next().await.expect( "1 connection" ).expect( "1 connection" ).await.expect( "WS handshake" );

		let mut sink = LinesCodec::new().framed( server ).sink_compat();

		sink.send( "A line"       .to_string() ).await.expect( "Send a line"        );
		sink.send( "A second line".to_string() ).await.expect( "Send a second line" );
	};

	let client = async move
	{
		debug!( "client task" );

		let client = WsStream::connect( "127.0.0.1:3112" ).await.expect( "connect to websocket" );

		let mut stream = LinesCodec::new().framed( client ).compat();

		let res = stream.next().await.expect( "Receive Some" ).expect( "Receive a line" );
		assert_eq!( "A line", dbg!( &res ) );

		let res = stream.next().await.expect( "Receive Some" ).expect( "Receive a line" );
		assert_eq!( "A second line", dbg!( &res ) );
	};

	rt::spawn( server ).expect( "spawn task" );
	rt::spawn( client ).expect( "spawn task" );
	rt::run();
}

