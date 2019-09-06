// Test using warp as a backend
//
// ✔ TungWebSocket::connect to a warp server
//


use
{
	ws_stream     :: { *                                                                              } ,
	futures       :: { StreamExt, SinkExt, future::{ FutureExt, TryFutureExt }, channel::oneshot      } ,
	futures_codec :: { LinesCodec, Framed                                                             } ,
	warp          :: { Filter                                                                         } ,
	std           :: { net::SocketAddr                                                                } ,
	tokio         :: { runtime::current_thread::Runtime                                               } ,

	// log           :: { * } ,
};


#[ test ]
//
fn warp()
{
	// flexi_logger::Logger::with_str( "warp=trace, ws_stream=trace, tokio=warn" ).start().expect( "flexi_logger");


	let (shutdown_tx, shutdown_rx) = oneshot::channel();

	let client = async
	{
		let     socket = TungWebSocket::connect( "127.0.0.1:3014" ).await.expect( "connect to websocket" );
		let     client = WsStream::new( socket );
		let mut framed = Framed::new( client, LinesCodec {} );


		let res = framed.next().await.expect( "Receive some" ).expect( "Receive a line" );
		assert_eq!( "A line\n".to_string(), res );


		let res = framed.next().await.expect( "Receive some" ).expect( "Receive a second line" );
		assert_eq!( "A second line\n".to_string(), res );


		let res = framed.next().await;
		dbg!( &res );
		assert!( res.is_none() );

		shutdown_tx.send(()).expect( "shutdown server" );
	};


	// GET /chat -> websocket upgrade
	//
	let chat = warp::path::end()

		// The `ws2()` filter will prepare Websocket handshake...
		//
		.and( warp::ws2() )


		.map( |ws: warp::ws::Ws2|
		{
			// This will call our function if the handshake succeeds.
			//
			ws.on_upgrade( move |socket|

				handle_conn( WarpWebSocket::new(socket) ).boxed().compat()
			)
		})
	;



	async fn handle_conn( socket: WarpWebSocket ) -> Result<(), ()>
	{
		let server = WsStream::new( socket );

		let mut framed = Framed::new( server, LinesCodec {} );

		framed.send( "A line\n"       .to_string() ).await.expect( "Send a line" );
		framed.send( "A second line\n".to_string() ).await.expect( "Send a line" );
		framed.close().await.expect( "close frame" );

		Ok(())
	}


	let addr: SocketAddr = "127.0.0.1:3014".to_string().parse().expect( "valid addr" );

	let mut runtime = Runtime::new().unwrap();

	let (_addr, server) = warp::serve( chat ).bind_with_graceful_shutdown( addr, shutdown_rx.compat() );

	runtime.spawn( client.unit_error().boxed().compat() );
	runtime.spawn( server );

	runtime.run().expect( "run runtime" );
}

