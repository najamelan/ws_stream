use crate::{ import::* };


#[derive(Debug, Eq, PartialEq, Clone)]
//
pub(crate) enum InnerMessage
{
	Tungstenite( TungMessage ),
	Warp       ( WarpMessage ),
}


/// An enum representing the various forms of a WebSocket message.
//
#[ derive( Debug, Eq, PartialEq, Clone ) ]
//
pub struct Message
{
	empty: [u8;0]       ,
	inner: InnerMessage ,
}


#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
#[ allow( missing_docs ) ]
//
pub enum MessageKind
{
	Text   ,
	Binary ,
	Ping   ,
	Pong   ,
	Close  ,
}



impl Message
{
	pub(crate) fn new( inner: InnerMessage ) -> Self
	{
		Self
		{
			empty: [],
			inner
		}
	}


	/// The kind of Websocket Message.
	//
	pub fn kind(&self) -> MessageKind
	{
		match self.inner
		{
			InnerMessage::Tungstenite(ref inner) =>
			{
				match inner
				{
					TungMessage::Text  (_) => MessageKind::Text   ,
					TungMessage::Binary(_) => MessageKind::Binary ,
					TungMessage::Ping  (_) => MessageKind::Ping   ,
					TungMessage::Pong  (_) => MessageKind::Pong   ,
					TungMessage::Close (_) => MessageKind::Close  ,
				}
			}

			InnerMessage::Warp(ref inner) =>
			{
				if      inner.is_text  () { return MessageKind::Text   }
				else if inner.is_binary() { return MessageKind::Binary }
				else if inner.is_ping  () { return MessageKind::Ping   }

				// Missing from warp API
				// TODO: check https://github.com/seanmonstar/warp/issues/257
				//
				// else if inner.is_pong  () { return MessageKind::Pong   }

				else if inner.is_close () { return MessageKind::Close   }

				// For now, assume that if it's nothing else, it's a pong, although
				// normally tungstenite swallows these.
				//
				else { return MessageKind::Pong }
			}
		}
	}


	/// Get a reference to underlying data
	//
	pub fn as_bytes(&self) -> &[u8]
	{
		match self.inner
		{
			InnerMessage::Tungstenite(ref inner) =>
			{
				match inner
				{
					TungMessage::Text  (t) => t.as_bytes()                ,

					TungMessage::Binary(d) |
					TungMessage::Ping  (d) |
					TungMessage::Pong  (d) => &d                          ,

					TungMessage::Close (None)     => &self.empty          ,
					TungMessage::Close (Some(cf)) => cf.reason.as_bytes() ,
				}
			}

			InnerMessage::Warp(ref inner) => inner.as_bytes(),
		}
	}


	/// Get the close frame from tungstenite if available. Note that Warp will currently swallow this.
	//
	pub fn close_frame( &self ) -> Option<&CloseFrame>
	{
		match self.inner
		{
			InnerMessage::Tungstenite(ref inner) =>
			{
				match inner
				{
					TungMessage::Close (Some(cf)) => Some( cf ),
					_                             => None      ,
				}
			}

			InnerMessage::Warp(_) => None,
		}
	}
}



impl From<TungMessage> for Message
{
	fn from( inc: TungMessage ) -> Self
	{
		Self::new( InnerMessage::Tungstenite(inc) )
	}
}



impl From<WarpMessage> for Message
{
	fn from( inc: WarpMessage ) -> Self
	{
		Self::new( InnerMessage::Warp(inc) )
	}
}



impl From<Message> for TungMessage
{
	/// Convert the message to a Tungstenite message. If it was a warp message, this will copy data
	//
	fn from( inc: Message ) -> Self
	{
		let kind = inc.kind();

		match inc.inner
		{
			InnerMessage::Tungstenite(inner) => inner,

			InnerMessage::Warp(inner) =>
			{
				match kind
				{
					MessageKind::Text   => TungMessage::Text  ( inner.to_str().expect( "get str from warp message").to_owned() ),
					MessageKind::Binary => TungMessage::Binary( inner.as_bytes().to_owned() ),

					// Close frame swallowed by warp
					//
					MessageKind::Close  => TungMessage::Close ( None ),

					// This shouldn't happen because tungstenite automatically responds to pings and swallows them
					// TODO: test trying to send a ping ourselves to be sure tungstenites swallows the pong.
					//
					MessageKind::Ping   |
					MessageKind::Pong   => unreachable!(),
				}
			}
		}
	}
}



impl From<Message> for WarpMessage
{
	fn from( inc: Message ) -> Self
	{
		match inc.inner
		{
			InnerMessage::Warp(inner) => inner,

			InnerMessage::Tungstenite(inner) =>
			{
				match inner
				{
					TungMessage::Text(s)   => WarpMessage::text  ( s ),
					TungMessage::Binary(v) => WarpMessage::binary( v ),

					// Close frame swallowed by warp
					//
					TungMessage::Close(_)  => unimplemented!(),

					// This shouldn't happen because tungstenite automatically responds to pings and swallows them
					// TODO: test trying to send a ping ourselves to be sure tungstenites swallows the pong.
					//
					TungMessage::Ping(_)   |
					TungMessage::Pong(_)   => unreachable!(),
				}
			}
		}
	}
}
