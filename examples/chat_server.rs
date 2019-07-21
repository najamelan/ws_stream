//! This is an echo server that returns all incoming bytes, without framing. It is used for the tests in
//! ws_stream_wasm.
//
#![ feature( async_await, async_closure ) ]


use
{
	chat_format   :: { futures_serde_cbor::Codec, Wire, ClientMsg, ServerMsg } ,
	log           :: { *                                                            } ,
	ws_stream     :: { *                                                            } ,
	async_runtime :: { rt, RtConfig                                                 } ,
	std           :: { env, cell::RefCell, collections::HashMap, net::SocketAddr    } ,
	futures_codec :: { Framed                                                       } ,
	// rand          :: { thread_rng, Rng, distributions::Alphanumeric              } ,

	futures ::
	{
		StreamExt                                     ,
		compat::Compat01As03                          ,
		channel::mpsc::{ unbounded, UnboundedSender } ,
		sink::SinkExt                                 ,
	},
};


type ConnMap = RefCell< HashMap<SocketAddr, UnboundedSender<Wire>> >;


static WELCOME : &str   = "Welcome to the ws_stream Chat Server!";

thread_local!
{
	static CONNS : ConnMap        = RefCell::new( HashMap::new() );
	static CLIENT: RefCell<usize> = RefCell::new( 0 );
}


fn main()
{
	flexi_logger::Logger::with_str( "echo=trace, ws_stream=error, tokio=warn" ).start().unwrap();

	// We only need one thread.
	//
	rt::init( RtConfig::Local ).expect( "init rt" );


	let server = async move
	{
		let addr: String = env::args().nth(1).unwrap_or( "127.0.0.1:3412".to_string() ).parse().unwrap();
		println!( "server task listening at: {}", &addr );

		WsStream::listen( &addr ).for_each_concurrent( None, handle_conn ).await;
	};


	rt::spawn_local( server ).expect( "spawn task" );
	rt::run();
}



async fn handle_conn( stream: Result<Compat01As03<Accept>, WsErr> )
{
	let ws_stream = stream.expect( "tcp stream" ).await.expect( "ws handshake" );
	let peer_addr = ws_stream.peer_addr().expect( "peer addr" );

	println!( "Incoming connection from: {}", peer_addr );

	let (tx, rx)            = unbounded();
	let framed              = Framed::new( ws_stream, Codec::new() );
	let (mut out, mut msgs) = framed.split();
	let mut nick            = peer_addr.to_string();

	// A unique sender id for this client
	//
	let sid: usize          = CLIENT.with( |cnt| { *cnt.borrow_mut() += 1; cnt.borrow().clone() } );


	// Welcome message
	//
	println!( "sending welcome line" );

	out.send( Wire::Server( ServerMsg::ServerMsg( WELCOME.to_string() ) ) ).await.expect( "send welcome" );

	CONNS.with( |conns| conns.borrow_mut().insert( peer_addr, tx ) );




	let outgoing = async move
	{
		// Send out to the client all messages that come in over the channel
		//
		match rx.map( |res| Ok( res ) ).forward( out ).await
		{
			Err(e) =>
			{
				CONNS.with( |conns| conns.borrow_mut().remove( &peer_addr ) );

				info!( "Client disconnected: {}", peer_addr );
				error!( "{}", e );
			},

			Ok(_)  => {}
		};
	};


	rt::spawn_local( outgoing ).expect( "spawn outgoing" );




	// Incoming messages
	//
	while let Some( msg ) = msgs.next().await
	{
		// TODO: handle io errors
		//
		let msg = match msg
		{
			Ok( Wire::Client( msg ) ) => msg,
			_                         => continue,
		};


		match msg
		{
			ClientMsg::SetNick( new_nick ) =>
			{
				broadcast( &ServerMsg::ServerMsg( format!( "{} changed nick => {}\n", &nick, &new_nick ) ) );
				nick = new_nick;
			}


			ClientMsg::ChatMsg( txt ) =>
			{
				broadcast( &ServerMsg::ChatMsg { nick: nick.clone(), sid, txt } );
			}
		}
	};
}



// Send a server message to all connected clients
//
fn broadcast( msg: &ServerMsg )
{
	CONNS.with( |conns|
	{
		let conns = conns.borrow();

		for client in conns.values()
		{
			client.unbounded_send( Wire::Server( msg.clone() ) ).expect( "send on unbounded" );
		};
	});
}



// fn random_id() -> String
// {
// 	thread_rng()
// 		.sample_iter( &Alphanumeric )
// 		.take(8)
// 		.collect()
// }

