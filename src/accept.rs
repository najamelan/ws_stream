use
{
	crate :: { import::*, TungWebSocket, WsErr, WsErrKind } ,
};


/// Future representing a finished WS handshake.
//
pub struct Accept
{
	inner: AndThen
	<
		FutureResult< TcpStream, tungstenite::error::Error > ,
		AcceptAsync<TcpStream, NoCallback>                   ,
		fn(TcpStream) -> AcceptAsync<TcpStream, NoCallback>  ,
	>,

	peer: Option<SocketAddr>,
}


impl Accept
{
	/// Create a new accept handshake from an AsyncRead01/Write01
	//
	pub fn new( stream: TcpStream ) -> Self
	{
		Self
		{
			peer : stream.peer_addr().ok()              ,
			inner: ok( stream ).and_then( accept_async ),
		}
	}
}



// I think we tried making this a std future, but failed. I can't remember why.
//
impl Future01 for Accept
{
	type Item  = TungWebSocket<TcpStream>;
	type Error = WsErr                   ;

	fn poll( &mut self ) -> Result< Async< Self::Item >, Self::Error >
	{
		match self.inner.poll()
		{
			Ok ( Async::Ready   ( ws ) ) => { trace!( "accept ok"      ); Ok ( Async::Ready( TungWebSocket::new(ws, self.peer) ) ) },
			Ok ( Async::NotReady       ) => { trace!( "accept pending" ); Ok ( Async::NotReady                                   ) },
			Err( e                     ) => { error!( "{}", &e         ); Err( WsErrKind::WsHandshake.into()                     ) },
		}
	}
}
