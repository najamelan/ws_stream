
use
{
	ws_stream     :: { *                                             } ,
	futures       :: { SinkExt, future::{ FutureExt, TryFutureExt }  } ,
	futures_codec :: { LinesCodec, Framed                            } ,
	warp          :: { Filter                                        } ,
	std           :: { net::SocketAddr                               } ,
	tokio         :: { runtime::current_thread::Runtime              } ,

	// log           :: { * } ,
};


fn main()
{
	// flexi_logger::Logger::with_str( "warp=trace, ws_stream=trace, tokio=warn" ).start().expect( "flexi_logger");

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


	let addr: SocketAddr = "127.0.0.1:4443".to_string().parse().expect( "valid addr" );

	let mut runtime = Runtime::new().unwrap();

	let server = warp::serve( chat )

		.tls( "tests/cert/ws.stream.crt", "tests/cert/cert.key" )
		.bind( addr )
	;

	runtime.spawn( server );

	runtime.run().expect( "run runtime" );
}
