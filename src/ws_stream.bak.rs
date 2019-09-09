use crate::{ import::*, WsErr, WsErrKind, Incoming };


type AsyncTokioResult = Result< Async<()>, tokio::io::Error >;


#[derive(Debug, Clone)]
//
enum ReadState
{
	Ready { chunk: Vec<u8>, chunk_start: usize } ,
	PendingChunk                                 ,
}


/// Represents a duplex stream of bytes on top of a websocket connection. This type implements AsyncRead/Write
/// from both tokio and futures 0.3.
///
/// Convenience methods are provided for TCP as underlying stream, but it can work over any tokio-tungstenite
/// websocket, so on top of any tokio AsyncRead/AsyncWrite.
///
/// Both server (listen) and client (connect) are available, even though this library is mainly intended
/// to be the server end of wasm modules that need to communicate over an AsyncRead.
///
//
pub struct Connection<S: AsyncRead01 + AsyncWrite01>
{
	stream : SplitStream < WebSocketStream<S> >,
	sink   : SplitSink   < WebSocketStream<S> >,
	state  : ReadState                         ,
	peer   : Option< SocketAddr >              ,
}



impl Connection<TcpStream>
{
	/// Listen over tcp. This will return a stream of WsStream, one for each incoming connection.
	//
	pub fn listen< T: AsRef<str> >( url: T )  -> Incoming
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

			Err(e) => Err( e.context( WsErrKind::TcpConnection ).into() ),
		}
	}


	/// If the WsStream was created with a peer_addr set, you can retrieve it here.
	//
	pub fn peer_addr( &self ) -> Option<SocketAddr>
	{
		self.peer
	}
}



impl<S: AsyncRead01 + AsyncWrite01> Connection<S>
{
	/// Create a Connection
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
		trace!( "Connection: read called" );

		loop { match &mut self.state
		{
			ReadState::Ready { chunk, chunk_start } =>
			{
				trace!( "io_read: received message" );

				let end = cmp::min( *chunk_start + buf.len(), chunk.len() );
				let len = end - *chunk_start;

				buf[..len].copy_from_slice( &chunk[*chunk_start..end] );


				if chunk.len() == end
				{
					self.state = ReadState::PendingChunk;
				}

				else
				{
					*chunk_start = end;
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
						// Check for tungstenite::Message::Close and return EOF
						// TODO: provide observable events?
						//
						if let tungstenite::Message::Close( close_frame ) = chunk
						{
							match close_frame
							{
								Some(f) => trace!( "io_read: connection is closed by client with code: {} and reason: {}", f.code, f.reason ),
								None    => trace!( "io_read: connection is closed by client without code and reason." ),
							}

							return Ok( 0 );
						}

						// Otherwise transform it into a Vec<u8>
						//
						self.state = ReadState::Ready { chunk: chunk.into(), chunk_start: 0 };
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

						match err
						{
							tungstenite::error::Error::Io(e) => { return Err( e ) }

							// These can be connection closed, amongst others
							//
							_ => { return Ok(0) }
						}

					}
				}
			}
		}}
	}



	// -------io:Write impl
	//
	fn io_write( &mut self, buf: &[u8] ) -> io::Result< usize >
	{
		trace!( "Connection: io_write called" );

		// FIXME: avoid extra copy?
		//
		match self.sink.start_send( buf.into() )
		{
			Ok( AsyncSink::Ready       ) => { return Ok( buf.len() ); }
			Ok( AsyncSink::NotReady(_) ) => { trace!( "io_write: would block" ); return Err( io::Error::from( WouldBlock ) ) }

			Err(e) =>
			{
				match e
				{
					// The connection is closed normally, probably by the remote.
					//
					tungstenite::Error::ConnectionClosed =>
					{
						error!( "{}", e );
						return Err( io::Error::from( io::ErrorKind::NotConnected ) );
					}

					// Trying to work with an already closed connection.
					//
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
					// The connection is closed normally, probably by the remote.
					//
					tungstenite::Error::ConnectionClosed =>
					{
						error!( "{}", e );
						return Err( io::Error::from( io::ErrorKind::NotConnected ) );
					}

					// Trying to work with an already closed connection.
					//
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
		match self.sink.close()
		{
			Err(e) =>
			{
				error!( "{}", e );
			}

			_ => {}
		}

		Ok(().into())
	}
}



impl<S: AsyncRead01 + AsyncWrite01> Drop for Connection<S>
{
	fn drop( &mut self )
	{
		trace!( "Drop Connection" );

		let _ = self.async_shutdown();
	}
}






impl<S: AsyncRead01 + AsyncWrite01> io::Read  for Connection<S>
{
	fn read( &mut self, buf: &mut [u8] ) -> Result< usize, io::Error >
	{
		self.io_read( buf )
	}
}

impl<S: AsyncRead01 + AsyncWrite01> io::Read  for Pin< &mut Connection<S> >
{
	fn read( &mut self, buf: &mut [u8] ) -> Result< usize, io::Error >
	{
		self.io_read( buf )
	}
}


impl<S: AsyncRead01 + AsyncWrite01> io::Write for Connection<S>
{
	fn write( &mut self, buf: &[u8] ) -> io::Result< usize > { self.io_write( buf ) }
	fn flush( &mut self             ) -> io::Result< ()    > { self.io_flush(     ) }
}


impl<S: AsyncRead01 + AsyncWrite01> io::Write for Pin< &mut Connection<S> >
{
	fn write( &mut self, buf: &[u8] ) -> io::Result< usize > { self.io_write( buf ) }
	fn flush( &mut self             ) -> io::Result< ()    > { self.io_flush(     ) }
}


impl<S: AsyncRead01 + AsyncWrite01> AsyncRead01  for Connection<S>             {}
impl<S: AsyncRead01 + AsyncWrite01> AsyncRead01  for Pin< &mut Connection<S> > {}


impl<S: AsyncRead01 + AsyncWrite01> AsyncWrite01 for Connection<S>
{
	fn shutdown( &mut self ) -> AsyncTokioResult
	{
		self.async_shutdown()
	}
}


impl<S: AsyncRead01 + AsyncWrite01> AsyncWrite01 for Pin< &mut Connection<S> >
{
	fn shutdown( &mut self ) -> AsyncTokioResult
	{
		self.async_shutdown()
	}
}



impl<S: AsyncRead01 + AsyncWrite01> AsyncRead  for Connection<S>
{
	fn poll_read( self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8] ) -> Poll<Result<usize, io::Error>>
	{
		Pin::new( &mut AsyncRead01CompatExt::compat( self ) ).poll_read( cx, buf )
	}
}



impl<S: AsyncRead01 + AsyncWrite01> AsyncWrite for Connection<S>
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
		self.async_shutdown().expect( "shutdown sink for Connection" );
		Poll::Ready( Ok(()) )
	}
}
