use
{
	crate :: { import::*, WsStream, WsErr, WsErrKind } ,
};


/// Future representing a finished WS handshake.Accept
//
pub struct Accept<T: AsyncRead01 + AsyncWrite01 >
{
	inner: AndThen< FutureResult<T, tungstenite::error::Error>, AcceptAsync<T, NoCallback>, fn(T) -> AcceptAsync<T, NoCallback> >,
}


impl<T: AsyncRead01 + AsyncWrite01 > Accept<T>
{
	/// Create a new accept handshake from an AsyncRead01/Write
	//
	pub fn new( stream: T ) -> Self
	{
		Self{ inner: ok( stream ).and_then( accept_async ) }
	}
}



impl<T: AsyncRead01 + AsyncWrite01 > Future01 for Accept<T>
{
	type Item  = WsStream<T>;
	type Error = WsErr   ;

	fn poll( &mut self ) -> Result< Async< Self::Item >, Self::Error >
	{
		match self.inner.poll()
		{
			Ok ( Async::Ready   ( ws ) ) => { trace!( "accept ok"      ); Ok ( Async::Ready( WsStream::new(ws) ) ) },
			Ok ( Async::NotReady       ) => { trace!( "accept pending" ); Ok ( Async::NotReady                   ) },
			Err( e                     ) => { error!( "{}", &e         ); Err( WsErrKind::WsHandshake.into()     ) },
		}
	}
}
