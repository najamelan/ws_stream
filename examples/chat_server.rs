//! This is an echo server that returns all incoming bytes, without framing. It is used for the tests in
//! ws_stream_wasm.
//
#![ feature( async_await, async_closure ) ]


use
{
	chat_format   :: { futures_serde_cbor::Codec, ClientMsg, ServerMsg             } ,
	log           :: { *                                                                 } ,
	ws_stream     :: { *                                                                 } ,
	async_runtime :: { rt, RtConfig                                                      } ,
	std           :: { env, cell::RefCell, collections::HashMap, net::SocketAddr, rc::Rc } ,
	futures_codec :: { Framed                                                            } ,
	// rand          :: { thread_rng, Rng, distributions::Alphanumeric              } ,

	futures ::
	{
		StreamExt                                     ,
		compat::Compat01As03                          ,
		channel::mpsc::{ unbounded, UnboundedSender } ,
		sink::SinkExt                                 ,
	},
};


type ConnMap = RefCell< HashMap<SocketAddr, Connection> >;


struct Connection
{
	nick     : Rc<RefCell<String>>        ,
	sid      : usize                      ,
	tx       : UnboundedSender<ServerMsg> ,
}


static WELCOME : &str = "Welcome to the ws_stream Chat Server!";

thread_local!
{
	static CONNS : ConnMap        = RefCell::new( HashMap::new() );
	static CLIENT: RefCell<usize> = RefCell::new( 0 );
}


fn main()
{
	flexi_logger::Logger::with_str( "chat_server=trace, ws_stream=error, tokio=warn" ).start().unwrap();

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



// Runs once for each incoming connection, ends when the stream closes or sending causes an
// error.
//
async fn handle_conn( stream: Result<Compat01As03<Accept>, WsErr> )
{
	let ws_stream = stream.expect( "tcp stream" ).await.expect( "ws handshake" );
	let peer_addr = ws_stream.peer_addr().expect( "peer addr" );

	println!( "Incoming connection from: {}", peer_addr );

	let (tx, rx)            = unbounded();
	let framed              = Framed::new( ws_stream, Codec::new() );
	let (mut out, mut msgs) = framed.split();
	let nick                = Rc::new( RefCell::new( peer_addr.to_string() ) );

	// A unique sender id for this client
	//
	let sid: usize          = CLIENT.with( |cnt| { *cnt.borrow_mut() += 1; cnt.borrow().clone() } );


	// Let all clients know there is a new kid on the block
	//
	broadcast( &ServerMsg::UserJoined { nick: nick.borrow().to_string(), sid } );


	// Welcome message
	//
	println!( "sending welcome message" );

	let all_users = CONNS.with( |conns|
	{
		conns.borrow_mut().insert
		(
			peer_addr,
			Connection { tx, nick: nick.clone(), sid },
		);

		conns.borrow().values().map( |c| (c.sid, c.nick.borrow().to_string() )).collect()
	});

	out.send( ServerMsg::Welcome
	{
		txt  : WELCOME.to_string(),
		users: all_users,

	}).await.expect( "send welcome" );



	let nick2 = nick.clone();

	// Listen to the channel for this connection and sends out each message that
	// arrives on the channel.
	//
	let outgoing = async move
	{
		match rx.map( |res| Ok( res ) ).forward( out ).await
		{
			Err(e) =>
			{
				let user = CONNS.with( |conns| conns.borrow_mut().remove( &peer_addr ) );

				if user.is_some()
				{
					// let other clients know this client disconnected
					//
					broadcast( &ServerMsg::UserLeft { nick: nick2.borrow().to_string(), sid } );


					debug!( "Client disconnected: {}", peer_addr );
					debug!( "{}", e );
				}

			},

			Ok(_)  => {}
		};
	};


	rt::spawn_local( outgoing ).expect( "spawn outgoing" );




	// Incoming messages. Ends when stream returns None or an error.
	//
	while let Some( msg ) = msgs.next().await
	{
		// TODO: handle io errors
		//
		let msg = match msg
		{
			Ok( msg ) => msg,
			_         => continue,
		};


		match msg
		{
			ClientMsg::SetNick( new_nick ) =>
			{
				broadcast( &ServerMsg::NickChanged{ old: nick.borrow().to_string(), new: new_nick.clone(), sid } );
				*nick.borrow_mut() = new_nick;
			}


			ClientMsg::ChatMsg( txt ) =>
			{
				broadcast( &ServerMsg::ChatMsg { nick: nick.borrow().to_string(), sid, txt } );
			}
		}
	};


	// remove the client and let other clients know this client disconnected
	//
	let user = CONNS.with( |conns| conns.borrow_mut().remove( &peer_addr ) );

	if user.is_some()
	{
		// let other clients know this client disconnected
		//
		broadcast( &ServerMsg::UserLeft { nick: nick.borrow().to_string(), sid } );


		debug!( "Client disconnected: {}", peer_addr );
	}

}



// Send a server message to all connected clients
//
fn broadcast( msg: &ServerMsg )
{
	CONNS.with( |conns|
	{
		let conns = conns.borrow();

		for client in conns.values().map( |c| &c.tx )
		{
			client.unbounded_send( msg.clone() ).expect( "send on unbounded" );
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

