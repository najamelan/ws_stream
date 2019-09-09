use crate:: { import::*, Message, WsErr, WsErrKind, Incoming };


/// A wrapper around a WebSocket provided by tungstenite.
/// The purpose of these providers is to deliver a unified interface for to higher level
/// code abstracting out different implementations of the websocket handshake and protocol.
//
pub struct TungWebSocket<S: AsyncRead01 + AsyncWrite01>
{
	sink  : Compat01As03Sink< SplitSink01  < TTungSocket<S> >, TungMessage >,
	stream: Compat01As03    < SplitStream01< TTungSocket<S> >              >,
	peer  : Option< SocketAddr >                                            ,
}


impl<S: AsyncRead01 + AsyncWrite01> TungWebSocket<S>
{
	/// Create a new Wrapper for a WebSocket provided by Tungstenite
	//
	pub fn new( inner: TTungSocket<S>, peer: Option<SocketAddr> ) -> Self
	{
		let (tx, rx) = inner.split();

		Self { stream: Compat01As03::new( rx ), sink: Compat01As03Sink::new( tx ), peer }
	}


	/// If the WsStream was created with a peer_addr set, you can retrieve it here.
	//
	pub fn peer_addr( &self ) -> Option<SocketAddr>
	{
		self.peer
	}
}



impl TungWebSocket<TcpStream>
{
	/// Listen over tcp. This will return a stream of [Accept], one for each incoming connection. Accept will
	/// resolve once the websocket handshake is complete. The echo example shows how to use this.
	//
	pub fn listen< T: AsRef<str> >( url: T ) -> Incoming
	{
		let addr = url.as_ref().parse().unwrap();

		// Create the event loop and TCP listener we'll accept connections on.
		//
		let socket = TcpListener::bind( &addr ).unwrap();
		info!( "Listening on: {}", addr );

		Incoming::new( socket.incoming().compat() )
	}


	/// Connect to a websocket over tcp.
	//
	pub async fn connect< T: AsRef<str> >( url: T ) -> Result< Self, WsErr >
	{
		let addr = url.as_ref().parse().expect( "parse socketaddress" );

		// Create the event loop and TCP listener we'll accept connections on.
		//
		let socket = TcpStream::connect( &addr ).compat().await;
		info!( "WsStream: connecting to: {}", addr );

		let url = "ws://".to_string() + url.as_ref() + "/";

		match socket
		{
			Ok(tcpstream) =>
			{
				// We need the ok and then because client_async does io outside of the future it returns.
				// See: https://github.com/snapview/tokio-tungstenite/issues/55
				//
				match ok(()).and_then( |_| { client_async( Url::parse( &url ).expect( "parse url" ), tcpstream ) } ).compat().await
				{
					Ok (ws) => Ok( Self::new( ws.0, Some( addr ) ) ),
					Err(e ) =>
					{
						error!( "{}", &e );
						Err( WsErrKind::WsHandshake.into() )
					},
				}
			}

			Err(e) => Err( WsErr { kind: WsErrKind::TcpConnection, inner: Some( Box::new( e ) ) } ),
		}
	}
}




impl TungWebSocket<MaybeTlsStream<TcpStream>>
{
	// /// Listen over tcp. This will return a stream of WsStream, one for each incoming connection.
	// //
	// pub fn listen< T: AsRef<str> >( url: T )  -> Incoming
	// {
	// 	let addr = url.as_ref().parse().unwrap();

	// 	// Create the event loop and TCP listener we'll accept connections on.
	// 	//
	// 	let socket = TcpListener::bind( &addr ).unwrap();
	// 	info!( "Listening on: {}", addr );

	// 	Incoming::new( socket.incoming().compat() )
	// }


	/// Connect to a websocket over tcp.
	//
	pub async fn connect_ssl< T: AsRef<str> >( url: T, domain: &str ) -> Result< Self, WsErr >
	{
		let addr = url.as_ref().parse().expect( "parse socketaddress" );

		// Create the event loop and TCP listener we'll accept connections on.
		//
		let socket = TcpStream::connect( &addr ).compat().await;
		info!( "WsStream: connecting to: {}", addr );

		let url = "wss://".to_string() + domain;

		match socket
		{
			Ok(tcpstream) =>
			{
				// We need the ok and then because client_async does io outside of the future it returns.
				// See: https://github.com/snapview/tokio-tungstenite/issues/55
				//
				match ok(()).and_then( |_| { client_async_tls( Url::parse( &url ).expect( "parse url" ), tcpstream ) } ).compat().await
				{
					Ok (ws) => Ok( Self::new( ws.0, Some( addr ) ) ),
					Err(e ) =>
					{
						error!( "{}", &e );
						Err( WsErrKind::WsHandshake.into() )
					},
				}
			}

			Err(e) => Err( WsErr { kind: WsErrKind::TcpConnection, inner: Some( Box::new( e ) ) } ),
		}
	}
}



impl<S: AsyncRead01 + AsyncWrite01> Stream for TungWebSocket<S>
{
	type Item = Result<Message, WsErr>;


	/// When returning an error, this will return an error from Tung. The only thing we can know about the
	/// inner Tungstenite error is the display string. There will be no error type to match. So we just return
	/// `WsErrKind::TungError` including the string.
	//
	fn poll_next( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll< Option<Self::Item> >
	{
		let res = ready!( Pin::new( &mut self.stream ).poll_next( cx ) );

		match res
		{
			None             => Poll::Ready( None )                  ,
			Some(Ok ( msg )) => Poll::Ready( Some(Ok( msg.into() )) ),
			Some(Err( err )) =>
			{
				// Let's see what information we can get from the debug formatting.
				//
				error!( "TungErr: {:?}", err );

				Poll::Ready(Some(Err( err.into() )))
			}
		}
	}
}



impl<S: AsyncRead01 + AsyncWrite01> Sink<Message> for TungWebSocket<S>
{
	type Error = WsErr;


	fn poll_ready( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.sink ).poll_ready( cx ).map_err( |e| e.into() )
	}


	fn start_send( mut self: Pin<&mut Self>, item: Message ) -> Result<(), Self::Error>
	{
		trace!( "TungWebSocket: start_send" );
		Pin::new( &mut self.sink ).start_send( item.into() ).map_err( |e| e.into() )
	}


	fn poll_flush( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "TungWebSocket: poll_flush" );

		Pin::new( &mut self.sink ).poll_flush( cx ).map_err( |e| e.into() )
	}


	fn poll_close( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::Error>>
	{
		trace!( "TungWebSocket: poll_close" );
		Pin::new( &mut self.sink ).poll_close( cx ).map_err( |e| e.into() )
	}
}


