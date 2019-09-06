use crate::{ import::*, WsErr, Message, MessageKind, WsErrKind, /*Incoming*/ };



#[derive(Debug, Clone)]
//
enum ReadState
{
	Ready { msg: Message, chunk_start: usize } ,
	PendingChunk                                 ,
}



/// Trait bounds
//
pub trait MsgStream: Stream<Item=Result<Message, WsErr>> + Sink<Message, Error=WsErr> {}

impl<S> MsgStream for S

	where S: Stream<Item=Result<Message, WsErr>> + Sink<Message, Error=WsErr>,

{}



/// takes a Stream + Sink of websocket messages and implements AsyncRead + AsyncWrite
//
pub struct WsStream<Inner: MsgStream>
{
	stream : SplitStream< Inner >          ,
	sink   : SplitSink  < Inner, Message > ,
	state  : ReadState                     ,
}



impl<Inner: MsgStream> WsStream<Inner>
{
	/// Create a WsStream
	//
	pub fn new( ws: Inner ) -> Self
	{
		let (sink, stream) = ws.split();

		Self{ stream, sink, state: ReadState::PendingChunk }
	}
}



impl<Inner: MsgStream> WsStream<Inner>
{
	//---------- impl io::Read
	//
	fn poll_read_impl( &mut self, cx: &mut Context<'_>, buf: &mut [u8] ) -> Poll< io::Result<usize> >
	{
		trace!( "WsStream: read called" );

		loop { match &mut self.state
		{
			ReadState::Ready { msg, chunk_start } =>
			{
				trace!( "io_read: received message" );

				let chunk = msg.as_bytes();

				let end = cmp::min( *chunk_start + buf.len(), chunk.len() );
				let len = end - *chunk_start;

				buf[..len].copy_from_slice( &chunk[*chunk_start..end] );


				// We read the entire chunk
				//
				if chunk.len() == end { self.state = ReadState::PendingChunk }
				else                  { *chunk_start = end                   }

				trace!( "io_read: return read {}", len );

				return Poll::Ready( Ok(len) );
			}


			ReadState::PendingChunk =>
			{
				trace!( "io_read: pending" );

				match Pin::new( &mut self.stream ).poll_next(cx)
				{
					// We have a message
					//
					Poll::Ready(Some(Ok( msg ))) =>
					{
						// Check for tungstenite::Message::Close and return EOF
						// TODO: provide observable events?
						//
						if msg.kind() == MessageKind::Close
						{
							match msg.close_frame()
							{
								Some(f) => trace!( "io_read: connection is closed by client with code: {} and reason: {}", f.code, f.reason ),
								None    => trace!( "io_read: connection is closed by client without code and reason." ),
							}

							return Poll::Ready( Ok(0) );
						}



						// Otherwise transform it into a Vec<u8>
						//
						self.state = ReadState::Ready { msg, chunk_start: 0 };
						continue;
					}

					// The stream has ended
					//
					Poll::Ready( None ) =>
					{
						trace!( "io_read: stream has ended" );
						return Poll::Ready( Ok(0) );
					}

					// No chunk yet, save the task to be woken up
					//
					Poll::Pending =>
					{
						trace!( "io_read: stream would_block" );

						return Poll::Pending;
					}

					Poll::Ready(Some( Err(err) )) =>
					{
						error!( "{}", err );

						return Poll::Ready(Err( Self::to_io_error(err) ))
					}
				}
			}
		}}
	}



	// -------io:Write impl
	//
	fn poll_write_impl( &mut self, cx: &mut Context, buf: &[u8] ) -> Poll< io::Result<usize> >
	{
		trace!( "WsStream: poll_write_impl called" );

		let res = ready!( Pin::new( &mut self.sink ).poll_ready(cx) );

		if let Err( e ) = res
		{
			trace!( "WsStream: poll_write_impl SINK not READY" );

			return Poll::Ready(Err( Self::to_io_error(e) ))
		}


		// FIXME: avoid extra copy?
		// The type of our sink is Message, but to create that you always have to decide whether
		// it's a TungMessage or a WarpMessage. Since converting from WarpMessage to TungMessage requires a
		// copy, we create it from TungMessage.
		// TODO: create a constructor on Message that automatically defaults to TungMessage here.
		//
		match Pin::new( &mut self.sink ).start_send( TungMessage::Binary( buf.into() ).into() )
		{
			Ok (_) =>
			{
				// The Compat01As03Sink always keeps one item buffered. Also, client code like
				// futures-codec and tokio-codec turn a flush on their sink in a poll_write here.
				// Combinators like CopyBufInto will only call flush after their entire input
				// stream is exhausted.
				// We actually don't buffer here, but always create an entire websocket message from the
				// buffer we get in poll_write, so there is no reason not to flush here, especially
				// since the sink will always buffer one item until flushed.
				// This means the burden is on the caller to call with a buffer of sufficient size
				// to avoid perf problems, but there is BufReader and BufWriter in the futures library to
				// help with that if necessary.
				//
				// We will ignore the Pending return from the flush, since we took the data and
				// must return how many bytes we took. The client should not try to send this data again.
				// This does mean there might be a spurious wakeup, TODO: we should test that.
				// We could supply a dummy context to avoid the wakup.
				//
				// So, flush!
				//
				let _ = Pin::new( &mut self.sink ).poll_flush( cx );

				trace!( "WsStream: poll_write_impl, wrote {} bytes", buf.len() );

				Poll::Ready(Ok ( buf.len() ))
			}

			Err(e) => Poll::Ready(Err( Self::to_io_error(e) )),
		}
	}



	fn poll_flush_impl( &mut self, cx: &mut Context ) -> Poll< io::Result<()> >
	{
		trace!( "flush AsyncWrite" );

		match ready!( Pin::new( &mut self.sink ).poll_flush(cx) )
		{
			Ok (_) => Poll::Ready(Ok ( ()                     )) ,
			Err(e) => Poll::Ready(Err( Self::to_io_error( e ) )) ,
		}
	}


	fn poll_close_impl( &mut self, cx: &mut Context ) -> Poll< io::Result<()> >
	{
		trace!( "Closing AsyncWrite" );

		match Pin::new( &mut self.sink ).poll_close( cx )
		{
			Poll::Ready( Err(e) ) =>
			{
				error!( "{}", e );
				Poll::Ready( Err(Self::to_io_error(e)) )
			}

			_ => Ok(()).into(),
		}
	}



	fn to_io_error( err: WsErr ) -> io::Error
	{
		error!( "{:?}", &err );

		match err.kind()
		{
			// This would be a tungstenite error, but since we can't access it through warp...
			//
			WsErrKind::WarpErr => return io::Error::from( io::ErrorKind::Other ),


			WsErrKind::TungErr =>
			{
				if let Some( e ) = err.source() { match e.downcast_ref::<TungErr>()
				{
					// The connection is closed normally, probably by the remote.
					//
					Some( TungErr::ConnectionClosed ) => return io::Error::from( io::ErrorKind::NotConnected      ) ,
					Some( TungErr::AlreadyClosed    ) => return io::Error::from( io::ErrorKind::ConnectionAborted ) ,
					Some( TungErr::Io(er)           ) => return io::Error::from( er.kind()                        ) ,
					_                                 => return io::Error::from( io::ErrorKind::Other             ) ,
				}}

				else { return io::Error::from( io::ErrorKind::Other ); }
			}

			WsErrKind::Protocol => io::Error::from( io::ErrorKind::ConnectionReset ),

			_ => unreachable!(),
		}
	}
}



impl<Inner: MsgStream> Drop for WsStream<Inner>
{
	fn drop( &mut self )
	{
		trace!( "Drop WsStream" );
	}
}




impl<Inner: MsgStream> AsyncRead  for WsStream<Inner>
{
	fn poll_read( mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8] ) -> Poll< io::Result<usize> >
	{
		self.poll_read_impl( cx, buf )
	}
}



impl<Inner: MsgStream> AsyncWrite for WsStream<Inner>
{
	// TODO: on WouldBlock, we should wake up the task when it becomes ready.
	//
	fn poll_write( mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8] ) -> Poll< io::Result<usize> >
	{
		trace!( "WsStream: AsyncWrite - poll_write" );
		self.poll_write_impl( cx, buf )
	}


	fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll< io::Result<()> >
	{
		trace!( "WsStream: AsyncWrite - poll_flush" );
		self.poll_flush_impl(cx)
	}


	fn poll_close( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll< io::Result<()> >
	{
		self.poll_close_impl( cx )
	}
}
