//! Provide AsyncRead/AsyncWrite over WebSockets (both tokio and 0.3)
//!
#![ doc    ( html_root_url = "https://docs.rs/ws_stream" ) ]
#![ feature( async_await                                 ) ]
#![ deny   ( missing_docs                                ) ]
#![ forbid ( unsafe_code                                 ) ]
#![ allow  ( clippy::suspicious_else_formatting          ) ]



mod accept      ;
mod connections ;
mod error       ;
mod ws_stream   ;

pub use
{
	accept          :: * ,
	connections     :: * ,
	error           :: * ,
	self::ws_stream :: * ,
};




mod import
{
	pub(crate) use
	{
		failure           :: { Backtrace, Fail, Context as FailContext                                                         } ,
		futures_01        :: { stream::{ SplitStream, SplitSink, Stream as _ }, AsyncSink, Future as Future01,                 } ,
		futures_01        :: { future::{ FutureResult, AndThen, ok }, sink::Sink as _                                          } ,
		futures::compat   :: { Stream01CompatExt, Future01CompatExt, AsyncWrite01CompatExt, AsyncRead01CompatExt, Compat01As03 } ,
		futures           :: { prelude::{ Stream, AsyncRead, AsyncWrite }, Poll, task::Context                                 } ,
		log               :: { info, trace, error                                                                              } ,
		std               :: { cmp::{ self }, io::{ self, ErrorKind::WouldBlock }, pin::Pin, fmt, net::SocketAddr              } ,
		tokio             :: { net::{ tcp::Incoming, TcpListener, TcpStream }                                                  } ,
		tokio             :: { io::{ AsyncRead as AsyncRead01, AsyncWrite as AsyncWrite01 }, prelude::{ Async }                } ,
		tokio_tungstenite :: { accept_async, client_async, WebSocketStream, AcceptAsync                                        } ,
		tungstenite       :: { Message, handshake::{ server::NoCallback }                                                      } ,
		url               :: { Url                                                                                             } ,
	};
}

