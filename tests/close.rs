// Test how connections get closed
//
// ✔ close without code and reason from client
// - close with close codes and reason string
// - test closing from server
// - test closing from client
// - error handling
//



use
{
	ws_stream     :: { *                                                            } ,
	futures       :: { StreamExt, SinkExt, executor::LocalPool, task::LocalSpawnExt } ,
	futures_codec :: { LinesCodec, Framed                                           } ,
	log           :: { * } ,
};


// #[ test ]
//
fn close_client()
{
	// flexi_logger::Logger::with_str( "close=trace, futures_codec=trace, ws_stream=trace, tokio=warn, tokio_tungstenite=trace, tungstenite=trace" ).start().expect( "flexi_logger");

	let mut pool     = LocalPool::new();
	let mut spawner  = pool.spawner();


	let server = async
	{
		let mut connections = TungWebSocket::listen( "127.0.0.1:3022" ).take(1);
		let     socket      = connections.next().await.expect( "1 connection" ).expect( "1 connection" ).await.expect( "WS handshake" );

		let server = WsStream::new( socket );
		let mut framed = Framed::new( server, LinesCodec {} );

		let res = framed.next().await;

		// We should just receive the close
		//
		assert!( res.is_none() );
	};


	let client = async
	{
		let     socket = TungWebSocket::connect( "127.0.0.1:3022" ).await.expect( "connect to websocket" );
		let     client = WsStream::new( socket );
		let mut framed = Framed::new( client, LinesCodec {} );

		info!( "calling close" );
		framed.close().await.expect( "close" );

		info!( "trying to send" );
		let res = framed.send( "a line\n".to_string() ).await;

		assert_eq!( std::io::ErrorKind::NotConnected, res.unwrap_err().kind() );
	};

	spawner.spawn_local( server ).expect( "spawn server" );
	spawner.spawn_local( client ).expect( "spawn client" );

	pool.run();
}
