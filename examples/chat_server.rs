//! This is an echo server that returns all incoming bytes, without framing. It is used for the tests in
//! ws_stream_wasm.
//
#![ feature( async_await, async_closure ) ]


use
{
	ws_stream     :: { *                                                         } ,
	async_runtime :: { rt, RtConfig                                              } ,
	std           :: { env, cell::RefCell, collections::HashMap, net::SocketAddr } ,
	futures_codec :: { LinesCodec, Framed                                        } ,

	futures ::
	{
		future::ready                                 ,
		StreamExt                                     ,
		compat::Compat01As03                          ,
		channel::mpsc::{ unbounded, UnboundedSender } ,
		sink::SinkExt                                 ,
	},
};


type ConnMap = RefCell< HashMap<SocketAddr, (String, UnboundedSender<String>)> >;

thread_local!
{
	static CONNS: ConnMap = RefCell::new( HashMap::new() );
}

fn main()
{
	// flexi_logger::Logger::with_str( "echo=trace, ws_stream=trace, tokio=warn" ).start().unwrap();

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

	let (tx, rx)        = unbounded();
	let framed          = Framed::new( ws_stream, LinesCodec {} );
	let (mut out, msgs) = framed.split();

	// Welcome message
	//
	out.send( "Welcome to the ws_stream chat server!".to_string() ).await.expect( "send welcome" );

	CONNS.with( |conns| conns.borrow_mut().insert( peer_addr, (peer_addr.to_string() + ": ", tx) ) );

	let inc = msgs.for_each( move |msg|
	{
		// TODO: handle io errors
		//
		let msg = msg.expect( "message" );

		// set the nick
		//
		if msg == "set nick"
		{
			// set the nick
		}

		else
		{
			CONNS.with( |conns|
			{
				let conns = conns.borrow();

				for (addr, data) in conns.iter()
				{
					if addr != &peer_addr
					{
						data.1.unbounded_send( data.0.clone() + &msg.clone() ).expect( "send on unbounded" );
					}
				};
			});
		}

		ready(())
	});

	rt::spawn_local( inc ).expect( "spawn inc" );

	// Send out to the client all messages that come in over the channel
	//
	rx.map( |res| Ok( res ) ).forward( out ).await.expect( "send on out" );
}

