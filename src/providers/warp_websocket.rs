use crate:: { import::*, Message, WsErr };


/// A wrapper around a WebSocket provided by warp.
/// The purpose of these providers is to deliver a unified interface for to higher level
/// code abstracting out different implementations of the websocket handshake and protocol.
//
pub struct WarpWebSocket
{
	sink  : Compat01As03Sink< SplitSink01  < WarpSocket >, WarpMessage >,
	stream: Compat01As03    < SplitStream01< WarpSocket >              >,
}


impl WarpWebSocket
{
	/// Create a new Wrapper for a WebSocket provided by warp
	//
	pub fn new( inner: WarpSocket ) -> Self
	{
		let (tx, rx) = inner.split();

		Self { stream: Compat01As03::new( rx ), sink: Compat01As03Sink::new( tx ) }
	}
}



impl Stream for WarpWebSocket
{
	type Item = Result<Message, WsErr>;


	/// When returning an error, this will return an error from warp. The only thing we can know about the
	/// inner Tungstenite error is the display string. There will be no error type to match. So we just return
	/// `WsErrKind::WarpError` including the string.
	//
	fn poll_next( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll< Option<Self::Item> >
	{
		let res = ready!( Pin::new( &mut self.stream ).poll_next( cx ) );

		match res
		{
			None             => Poll::Ready( None )                 ,
			Some(Ok ( msg )) => Poll::Ready(Some(Ok ( msg.into() ))),
			Some(Err( err )) =>
			{
				// Let's see what information we can get from the debug formatting.
				//
				error!( "WarpErr: {:?}", err );

				Poll::Ready(Some(Err( err.into() )))
			}
		}
	}
}



impl Sink<Message> for WarpWebSocket
{
	type Error = WsErr;


	fn poll_ready( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.sink ).poll_ready( cx ).map_err( |e| e.into() )
	}


	fn start_send( mut self: Pin<&mut Self>, item: Message ) -> Result<(), Self::Error>
	{
		Pin::new( &mut self.sink ).start_send( item.into() ).map_err( |e| e.into() )
	}


	fn poll_flush( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.sink ).poll_flush( cx ).map_err( |e| e.into() )
	}


	fn poll_close( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.sink ).poll_close( cx ).map_err( |e| e.into() )
	}
}
