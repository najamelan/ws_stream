//! Provide AsyncRead/AsyncWrite over WebSockets (both tokio and 0.3)
//!
#![ doc    ( html_root_url = "https://docs.rs/wasm_websocket_stream/0.1.0" ) ]
#![ feature( async_await                                                   ) ]
#![ deny   ( missing_docs                                                  ) ]
#![ forbid ( unsafe_code                                                   ) ]
#![ allow  ( clippy::suspicious_else_formatting                            ) ]



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
		failure      :: { Backtrace, Fail, Context as FailContext              } ,
		futures      :: { Poll                                     } ,
		futures      :: { compat ::{ Stream01CompatExt, Future01CompatExt, AsyncWrite01CompatExt, AsyncRead01CompatExt } } ,
		futures      :: { prelude::{ Stream, AsyncRead, AsyncWrite },           } ,
		tokio        :: { io::{ AsyncRead as AsyncRead01, AsyncWrite as AsyncWrite01 }, prelude::{ Async }           } ,
		std          :: { cmp::{ self }, io::{ self, ErrorKind::WouldBlock }  } ,
		std          :: { pin::Pin, fmt } ,
		log          :: { info, trace, error                                                 } ,
		tungstenite       :: { Message, handshake::{ server::NoCallback } } ,
		tokio             :: { net::{ tcp::Incoming, TcpListener, TcpStream }   } ,
		tokio_tungstenite :: { accept_async, client_async, WebSocketStream, AcceptAsync } ,
		futures           :: { compat::Compat01As03, task::Context } ,
		futures_01        :: { stream::{ SplitStream, SplitSink }, AsyncSink, Future as Future01, future::{ FutureResult, AndThen, ok } } ,
		url               :: { Url                                              } ,
	};
}

