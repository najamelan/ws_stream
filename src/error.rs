use crate::{ import::* };


/// The error type for errors happening in `ws_stream`.
///
/// Use [`WsErr::kind()`] to know which kind of error happened.
//
#[ derive( Debug ) ]
//
pub struct WsErr
{
	inner: FailContext<WsErrKind>,
}



/// The different kind of errors that can happen when you use the `ws_stream` API.
//
#[ derive( Clone, PartialEq, Eq, Debug, Fail ) ]
//
pub enum WsErrKind
{
	/// This is an error from tokio-tungstenite.
	//
	#[ fail( display = "The WebSocket handshake failed" ) ]
	//
	WsHandshake,

	/// An error happend on the tcp level when connecting.
	//
	#[ fail( display = "A tcp connection error happened" ) ]
	//
	TcpConnection,
}



impl Fail for WsErr
{
	fn cause( &self ) -> Option< &dyn Fail >
	{
		self.inner.cause()
	}

	fn backtrace( &self ) -> Option< &Backtrace >
	{
		self.inner.backtrace()
	}
}



impl fmt::Display for WsErr
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		fmt::Display::fmt( &self.inner, f )
	}
}


impl WsErr
{
	/// Allows matching on the error kind
	//
	pub fn kind( &self ) -> &WsErrKind
	{
		self.inner.get_context()
	}
}

impl From<WsErrKind> for WsErr
{
	fn from( kind: WsErrKind ) -> WsErr
	{
		WsErr { inner: FailContext::new( kind ) }
	}
}

impl From< FailContext<WsErrKind> > for WsErr
{
	fn from( inner: FailContext<WsErrKind> ) -> WsErr
	{
		WsErr { inner }
	}
}


