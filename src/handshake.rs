use
{
	crate :: { import::*, TungWebSocket, WsErr, WsErrKind } ,
};


/// Future resolving to a finished WS handshake.
/// This will resolve to a TungWebSocket<TcpStream> or a WsErrKind::WsHandshake error. This error
/// holds the underlying error from tungstenite which can be obtained with the `source` method.
//
pub struct Handshake
{
	inner: AndThen
	<
		FutureResult< TcpStream, tungstenite::error::Error > ,
		AcceptAsync<TcpStream, NoCallback>                   ,
		fn(TcpStream) -> AcceptAsync<TcpStream, NoCallback>  ,
	>,

	peer: Option<SocketAddr>,
}


impl Handshake
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



// I think I tried making this a std future, but failed. I can't remember why.
//
impl Future01 for Handshake
{
	type Item  = TungWebSocket<TcpStream>;
	type Error = WsErr                   ;

	fn poll( &mut self ) -> Result< Async< Self::Item >, Self::Error >
	{
		match self.inner.poll()
		{
			Ok ( Async::Ready   ( ws ) ) => { Ok ( Async::Ready( TungWebSocket::new(ws, self.peer) ) ) },
			Ok ( Async::NotReady       ) => { Ok ( Async::NotReady                                   ) },


			Err( e ) =>
			{
				Err( WsErr
				{
					kind : WsErrKind::WsHandshake ,
					inner: Some( Box::new( e ) )  ,
				})
			},
		}
	}
}
