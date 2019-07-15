use
{
	crate :: { import::*, WsErr, WsErrKind, Connections } ,
	futures_01::stream::Stream as _,
	futures_01::sink  ::Sink   as _,
};


// type AsyncIoResult    = Result< Async<usize>, io::Error        >;
type AsyncTokioResult = Result< Async<()>   , tokio::io::Error >;


#[derive(Debug, Clone)]
//
enum ReadState
{
	Ready { chunk: Message, chunk_start: usize },
	PendingChunk,
	Eof,
}


/// A tokio AsyncRead01/AsyncWrite01 representing a WebSocket connection. It only supports binary mode. Contrary to the rest of this library,
/// this will work on types from [futures 0.1](https://docs.rs/futures/0.1.25/futures/) instead of [0.3](https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.13/futures/index.html). This is because tokio currently is on futures 0.1, so the stream returned from
/// a codec will be 0.1.
///
/// Currently !Sync and !Send.
///
/// ## Example
///
/// This example if from the integration tests. Uses [tokio-serde-cbor](https://docs.rs/tokio-serde-cbor/0.3.1/tokio_serde_cbor/) to send arbitrary data that implements [serde::Serialize](https://docs.rs/serde/1.0.89/serde/trait.Serialize.html) over a websocket.
///
/// ```
/// ```
//
pub struct WsStream<S: AsyncRead01 + AsyncWrite01>
{
	stream : SplitStream < WebSocketStream<S> >,
	sink   : SplitSink   < WebSocketStream<S> >,
	state  : ReadState                         ,
	peer   : Option< SocketAddr >              ,
}



impl WsStream<TcpStream>
{
	/// Listen over tcp. This will return a stream of WsStream, one for each incoming connection.
	//
	pub fn listen< T: AsRef<str> >( url: T )  -> Connections
	{
		let addr = url.as_ref().parse().unwrap();

		// Create the event loop and TCP listener we'll accept connections on.
		// TODO, work on any AsyncRead01/AsyncWrite01 rather than only TCP.
		//
		let socket = TcpListener::bind( &addr ).unwrap();
		info!( "Listening on: {}", addr );

		Connections::new( socket.incoming().compat() )
	}


	/// Connect to a websocket over tcp.
	//
	pub async fn connect< T: AsRef<str> >( url: T ) -> Result< Self, WsErr >
	{
		let addr = url.as_ref().parse().expect( "parse socketaddress" );

		// Create the event loop and TCP listener we'll accept connections on.
		// TODO, work on any AsyncRead01/AsyncWrite01 rather than only TCP.
		//
		let socket = TcpStream::connect( &addr ).compat().await;
		info!( "Connecting to: {}", addr );

		let url = "ws://".to_string() + url.as_ref() + "/";

		match socket
		{
			Ok(tcpstream) =>
			{
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

			Err(_) => Err( WsErrKind::TcpConnection.into() ),
		}
	}


	/// If the WsStream was created with a peer_addr set, you can retrieve it here.
	//
	pub fn peer_addr( &self ) -> Option<SocketAddr>
	{
		self.peer
	}
}



impl<S: AsyncRead01 + AsyncWrite01> WsStream<S>
{
	/// Create a WsStream
	//
	pub fn new( ws: WebSocketStream<S>, peer: Option<SocketAddr> ) -> Self
	{
		let (sink, stream) = ws.split();

		Self{ stream, sink, state: ReadState::PendingChunk, peer }
	}


	//---------- impl io::Read
	//
	fn io_read( &mut self, buf: &mut [u8] ) -> io::Result< usize >
	{
		trace!( "WsStream: read called" );

		loop
		{
			// we need to be able to dereference the state to access the Vec<u8>
			//
			let state = self.state.clone();


			match state
			{
				ReadState::Ready { chunk, mut chunk_start } =>
				{
					trace!( "io_read: received message" );

					let chunk: Vec<u8> = ( chunk ).into();
					let len = cmp::min( buf.len(), chunk.len() - chunk_start );

					buf[..len].copy_from_slice( &chunk[chunk_start..chunk_start + len] );

					chunk_start += len;

					if chunk.len() == chunk_start
					{
						self.state = ReadState::PendingChunk;
					}

					else
					{
						self.state = ReadState::Ready{ chunk: Message::Binary( chunk ), chunk_start }
					}

					return Ok( len );
				}

				ReadState::PendingChunk =>
				{
					trace!( "io_read: pending" );

					match self.stream.poll()
					{
						// We have a message
						//
						Ok( Async::Ready( Some( chunk ) ) ) =>
						{
							self.state = ReadState::Ready { chunk, chunk_start: 0 };
							continue;
						}

						// The stream has ended
						//
						Ok( Async::Ready( None ) ) =>
						{
							trace!( "io_read: stream has ended" );
							return Ok( 0 );
						}

						// No chunk yet, save the task to be woken up
						//
						Ok( Async::NotReady ) =>
						{
							trace!( "io_read: stream would_block" );

							return Err( io::Error::from( WouldBlock ) );
						}

						Err(err) =>
						{
							error!( "{}", err );

							self.state = ReadState::Eof;

							match err
							{
								tungstenite::error::Error::Io(e) => { return Err( e ) }

								_ => { return Ok(0) }
							}

						}
					}
				}

				ReadState::Eof => { trace!( "io_read: EOF" ); return Ok(0); }
			}
		}
	}






	// -------io:Write impl
	//
	fn io_write( &mut self, buf: &[u8] ) -> io::Result< usize >
	{
		trace!( "WsStream: io_write called" );

		let len = buf.len();

		// FIXME: avoid extra copy?
		//
		match self.sink.start_send( Message::binary( buf ) )
		{
			Ok( AsyncSink::Ready ) =>
			{
				match self.io_flush()
				{
					Ok ( () ) => return Ok (len),
					Err( e  ) => { error!( "{}", e ); return Err( e ) },

				}
			}

			Ok( AsyncSink::NotReady(_) ) => { trace!( "io_write: would block" ); return Err( io::Error::from( WouldBlock ) ) }

			Err(e) => { error!( "{}", e ); return Err( io::Error::from( io::ErrorKind::Other ) ) }
		}
	}


	fn io_flush( &mut self ) -> io::Result<()>
	{
		trace!( "flush AsyncWrite" );

		match self.sink.poll_complete()
		{
			Ok ( Async::Ready(_) ) => { return Ok ( () )                                                               }
			Ok ( Async::NotReady ) => { trace!( "io_flush: would block" ); return Err( io::Error::from( WouldBlock ) ) }

			Err(e) =>
			{
				match e
				{
					// The connection is closed normally, probably by the remote
					//
					tungstenite::Error::ConnectionClosed =>
					{
						error!( "{}", e );
						return Err( io::Error::from( io::ErrorKind::ConnectionAborted ) );
					}

					tungstenite::Error::AlreadyClosed =>
					{
						error!( "{}", e );
						return Err( io::Error::from( io::ErrorKind::ConnectionAborted ) );
					}

					_ =>
					{
						error!( "{}", e );
						return Err( io::Error::from( io::ErrorKind::Other ) )
					}
				}
			}
		}
	}



	// -------AsyncWrite01 impl
	//
	fn async_shutdown( &mut self ) -> AsyncTokioResult
	{
		trace!( "Closing AsyncWrite" );

		// This can not throw normally, because the only errors the api
		// can return is if we use a code or a reason string, which we don't.
		//
		self.sink.close().expect( "close ws socket" );

		Ok(().into())
	}
}


impl<S: AsyncRead01 + AsyncWrite01> Drop for WsStream<S>
{
	fn drop( &mut self )
	{
		trace!( "Drop WsStream" );

		// This can not throw normally, because the only errors the api
		// can return is if we use a code or a reason string, which we don't.
		//
		self.sink.close().expect( "WsStream::drop - close ws socket" );
	}
}






impl<S: AsyncRead01 + AsyncWrite01> io::Read  for WsStream<S>
{
	fn read( &mut self, buf: &mut [u8] ) -> Result< usize, io::Error >
	{
		self.io_read( buf )
	}
}

impl<S: AsyncRead01 + AsyncWrite01> io::Read  for Pin< &mut WsStream<S> >
{
	fn read( &mut self, buf: &mut [u8] ) -> Result< usize, io::Error >
	{
		self.io_read( buf )
	}
}


impl<S: AsyncRead01 + AsyncWrite01> io::Write for WsStream<S>
{
	fn write( &mut self, buf: &[u8] ) -> io::Result< usize > { self.io_write( buf ) }
	fn flush( &mut self             ) -> io::Result< ()    > { self.io_flush(     ) }
}


impl<S: AsyncRead01 + AsyncWrite01> io::Write for Pin< &mut WsStream<S> >
{
	fn write( &mut self, buf: &[u8] ) -> io::Result< usize > { self.io_write( buf ) }
	fn flush( &mut self             ) -> io::Result< ()    > { self.io_flush(     ) }
}


impl<S: AsyncRead01 + AsyncWrite01> AsyncRead01  for WsStream<S>             {}
impl<S: AsyncRead01 + AsyncWrite01> AsyncRead01  for Pin< &mut WsStream<S> > {}


impl<S: AsyncRead01 + AsyncWrite01> AsyncWrite01 for WsStream<S>
{
	fn shutdown( &mut self ) -> AsyncTokioResult
	{
		self.async_shutdown()
	}
}


impl<S: AsyncRead01 + AsyncWrite01> AsyncWrite01 for Pin< &mut WsStream<S> >
{
	fn shutdown( &mut self ) -> AsyncTokioResult
	{
		self.async_shutdown()
	}
}



impl<S: AsyncRead01 + AsyncWrite01> AsyncRead  for WsStream<S>
{
	fn poll_read( self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8] ) -> Poll<Result<usize, io::Error>>
	{
		Pin::new( &mut AsyncRead01CompatExt::compat( self ) ).poll_read( cx, buf )
	}
}



impl<S: AsyncRead01 + AsyncWrite01> AsyncWrite for WsStream<S>
{
	// TODO: on WouldBlock, we should wake up the task when it becomes ready.
	//
	fn poll_write( self: Pin<&mut Self>, cx: &mut Context, buf: &[u8] ) -> Poll<Result<usize, io::Error>>
	{
		Pin::new( &mut AsyncWrite01CompatExt::compat( self ) ).poll_write( cx, buf )
	}



	fn poll_flush(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), io::Error>>
	{
		match self.io_flush()
		{
			Ok (())  => { Poll::Ready( Ok(()) ) }

			Err( e ) =>
			{
				match e.kind()
				{
					io::ErrorKind::WouldBlock => Poll::Pending,
					_                         => { error!( "{}", &e ); Poll::Ready( Err(e) ) }
				}
			}
		}
	}


	fn poll_close( mut self: Pin<&mut Self>, _cx: &mut Context ) -> Poll<Result<(), io::Error>>
	{
		// This is infallible normally
		//
		self.async_shutdown().expect( "shutdown sink for wsstream" );
		Poll::Ready( Ok(()) )
	}
}
