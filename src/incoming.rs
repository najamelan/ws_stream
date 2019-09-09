use crate::{ import::*, WsErr, WsErrKind, Handshake };


/// A stream of incoming connections. Will yield [Handshake].
/// Can return WsErrKind::TcpConnection if the tcp connection fails. This error type contains
/// an inner source error for more information.
//
pub struct Incoming
{
	incoming: Compat01As03<TokioIncoming>,
}


impl Incoming
{
	// See pin-utils documentation for safety:
	// To make using this macro safe, three things need to be ensured:
	//
   // - If the struct implements Drop, the drop method is not allowed to move the value of the field.
   // - If the struct wants to implement Unpin, it has to do so conditionally:
   //   The struct can only implement Unpin if the field's type is Unpin.
   // - The struct must not be #[repr(packed)].
   //
	pin_utils::unsafe_pinned!( incoming: Compat01As03<TokioIncoming> );


	/// A new Incoming stream
	//
	pub fn new( incoming: Compat01As03<TokioIncoming> ) -> Self
	{
		Self{ incoming }
	}
}


impl Stream for Incoming
{
	type Item = Result< Compat01As03<Handshake>, WsErr >;


	fn poll_next( self: Pin<&mut Self>, cx: &mut Context ) -> Poll< Option<Self::Item> >
	{
		match self.incoming().poll_next( cx )
		{
			Poll::Pending => Poll::Pending,

			Poll::Ready( incoming ) =>
			{
				match incoming
				{
					Some( Ok(conn) ) =>
					{
						Poll::Ready(Some(Ok( Handshake::new(conn).compat() )))
					}

					Some( Err(e) ) => Poll::Ready(Some(Err( WsErr
					{
						kind : WsErrKind::TcpConnection ,
						inner: Some( Box::new( e ) )    ,
					}))),

					None => Poll::Ready( None ),
				}
			}
		}
	}
}



