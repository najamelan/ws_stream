//! Provide AsyncRead/AsyncWrite over WebSockets (both tokio and 0.3)
//!
#![ doc    ( html_root_url = "https://docs.rs/ws_stream" ) ]
#![ deny   ( missing_docs                                ) ]
#![ forbid ( unsafe_code                                 ) ]
#![ allow  ( clippy::suspicious_else_formatting, unused_imports ) ]



mod accept     ;
// mod connection ;
mod error      ;
mod incoming   ;
mod message    ;
mod ws_stream  ;
mod providers  ;

pub use
{
	accept          :: * ,
	// connection      :: * ,
	error           :: * ,
	incoming        :: * ,
	message         :: * ,

	providers       :: { TungWebSocket } ,
	self::ws_stream :: * ,
};


#[cfg( feature = "warp" )]
//
pub use
{
	providers:: { WarpWebSocket } ,
};




mod import
{
	pub(crate) use
	{
		futures_01        :: { stream::{ SplitStream as SplitStream01, SplitSink as SplitSink01, Stream as Stream01 }, AsyncSink, Future as Future01,                   } ,
		futures_01        :: { future::{ FutureResult, AndThen, ok }, sink::Sink as Sink01, Poll as Poll01                      } ,
		futures::compat   :: { Stream01CompatExt, Future01CompatExt, AsyncWrite01CompatExt, AsyncRead01CompatExt, Compat01As03, Compat, CompatSink, Compat01As03Sink   } ,
		futures           :: { prelude::{ Stream, Sink, AsyncRead, AsyncWrite }, Poll, task::Context, ready, StreamExt, TryStreamExt, stream::{ SplitStream, SplitSink } } ,
		log               :: { info, trace, error                                                                                } ,
		std               :: { cmp::{ self }, io::{ self, ErrorKind::WouldBlock }, pin::Pin, fmt, net::SocketAddr, error::Error as StdError, ops::Deref } ,
		tokio             :: { net::{ tcp::Incoming as TokioIncoming, TcpListener, TcpStream }                                                  } ,
		tokio             :: { io::{ AsyncRead as AsyncRead01, AsyncWrite as AsyncWrite01 }, prelude::{ Async }                } ,
		tokio_tungstenite :: { accept_async, client_async, WebSocketStream as TTungSocket, AcceptAsync                           } ,
		tungstenite       :: { handshake::{ server::NoCallback }, Message as TungMessage, Error as TungErr, protocol::CloseFrame } ,
		url               :: { Url                                                                                               } ,
	};


	#[cfg( feature = "warp" )]
	//
	pub(crate) use
	{
		warp:: { Error as WarpErr, ws::{ Message as WarpMessage, WebSocket as WarpSocket } } ,
	};
}

