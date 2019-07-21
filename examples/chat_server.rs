//! This is an echo server that returns all incoming bytes, without framing. It is used for the tests in
//! ws_stream_wasm.
//
#![ feature( async_await, async_closure ) ]


use
{
	chat_format   :: { futures_serde_cbor::Codec, ChatMessage, Command           } ,
	log           :: { *                                                         } ,
	ws_stream     :: { *                                                         } ,
	async_runtime :: { rt, RtConfig                                              } ,
	std           :: { env, cell::RefCell, collections::HashMap, net::SocketAddr } ,
	futures_codec :: { Framed                                                    } ,
	// rand          :: { thread_rng, Rng, distributions::Alphanumeric              } ,

	futures ::
	{
		StreamExt                                     ,
		compat::Compat01As03                          ,
		channel::mpsc::{ unbounded, UnboundedSender } ,
		sink::SinkExt                                 ,
	},
};


type ConnMap = RefCell< HashMap<SocketAddr, UnboundedSender<ChatMessage>> >;

static WELCOME : &str   = "Welcome to the ws_stream Chat Server!";
static SERVERID:  usize = 0;

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
	let uniq_id: usize      = CLIENT.with( |cnt| { *cnt.borrow_mut() += 1; cnt.borrow().clone() } );


	// Welcome message
	//
	println!( "sending welcome line" );

	out.send( server_msg( WELCOME.to_string() ) ).await.expect( "send welcome" );

	CONNS.with( |conns| conns.borrow_mut().insert( peer_addr, tx ) );




	let outgoing = async move
	{
		// Send out to the client all messages that come in over the channel
		//
		match rx.map( |res| Ok( res ) ).forward( out ).await
		{
			Err(e) =>
			{
				CONNS.with( |conns|
				{
					let mut conns = conns.borrow_mut();

					conns.remove( &peer_addr );

				});

				info!( "Lost connection: {}", peer_addr );
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
		let mut msg: ChatMessage = msg.expect( "message" );

		// set the nick
		//
		if msg.cmd == Command::SetNick  &&  msg.txt.is_some()
		{
			let new_nick = msg.txt.unwrap();
			msg  = server_msg( format!( "{} changed nick => {}\n", &nick, &new_nick ) );
			nick = new_nick;
		}


		else if msg.cmd == Command::Message
		{
			msg.sid  = Some( uniq_id      );
			msg.nick = Some( nick.clone() );
		}


		println!( "received line from: {}", &nick );

		CONNS.with( |conns|
		{
			let conns = conns.borrow();

			for client in conns.values()
			{
				client.unbounded_send( msg.clone() ).expect( "send on unbounded" );
			};
		});

	};
}


fn server_msg( msg: String ) -> ChatMessage
{
	ChatMessage
	{
		cmd : Command::ServerMessage       ,
		txt : Some( msg )                  ,
		nick: Some( "Server".to_string() ) ,
		sid : Some( SERVERID )             ,
	}
}


// fn random_id() -> String
// {
// 	thread_rng()
// 		.sample_iter( &Alphanumeric )
// 		.take(8)
// 		.collect()
// }

