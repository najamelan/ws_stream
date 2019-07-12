#![ feature( async_await ) ]


#[ cfg(not( target_arch = "wasm32" )) ]
//
fn main()
{
	use
	{
		ws_stream     :: { *                 } ,
		async_runtime :: { rt, RtConfig      } ,
		futures       :: { StreamExt, SinkExt } ,
		futures       :: { compat::{ Stream01CompatExt, Sink01CompatExt } } ,
		futures_01    :: { stream::Stream as _ } ,
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

		let codec = LinesCodec::new();

		let (sink, stream) = codec.framed( server ).split();
		let mut sink       = sink.sink_compat();
		let _stream        = stream.compat();

		sink.send( "A line".to_string() ).await.expect( "Send a line" );
	};

	let client = async move
	{
		debug!( "client task" );

		let client = WsStream::connect( "127.0.0.1:3112" ).await.expect( "connect to websocket" );

		let codec = LinesCodec::new();

		let (sink, stream) = codec.framed( client ).split();
		let _sink          = sink.sink_compat();
		let mut stream     = stream.compat();

		let res = stream.next().await.expect( "Receive a line" ).expect( "Receive a line" );

		dbg!( &res );

		assert_eq!( "A line".to_string(), res );
	};

	rt::spawn( server ).expect( "spawn task" );
	rt::spawn( client ).expect( "spawn task" );
	rt::run();
}


#[ cfg( target_arch = "wasm32" ) ]
fn main(){}
