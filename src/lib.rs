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
	pub use
	{
		failure      :: { Backtrace, Fail, Context as FailContext              } ,
		futures      :: { channel::{ oneshot, mpsc::unbounded }, Poll                                     } ,
		futures      :: { compat ::{ Compat01As03Sink, Stream01CompatExt, Sink01CompatExt, Future01CompatExt, AsyncWrite01CompatExt, AsyncRead01CompatExt } } ,
		futures      :: { prelude::{ Stream, Sink, AsyncRead, AsyncWrite }, task::Waker, stream::StreamExt           } ,
		tokio        :: { io::{ AsyncRead as AsyncRead01, AsyncWrite as AsyncWrite01 }, prelude::{ Async, task }           } ,
		std          :: { cmp::{ self, min }, io::{ self, Read, ErrorKind::WouldBlock }, collections::VecDeque, future::Future  } ,
		std          :: { sync::{ Mutex, Arc }, rc::Rc, cell::{ RefCell, RefMut }, pin::Pin, convert::{ TryFrom, TryInto }, fmt } ,
		log          :: { debug, info, warn, trace, error                                                 } ,
		tungstenite       :: { Message, handshake::{ client::Request, server::NoCallback } } ,
		tokio             :: { net::{ tcp::Incoming, TcpListener, TcpStream }   } ,
		tokio_tungstenite :: { accept_async, client_async, WebSocketStream, AcceptAsync } ,
		futures           :: { stream::Map, compat::Compat01As03, task::Context } ,
		futures_01        :: { stream::{ SplitStream, SplitSink }, AsyncSink, Future as Future01, future::{ Lazy, lazy, FutureResult, AndThen, ok } } ,
		url               :: { Url                                              } ,
		std               :: { marker::PhantomData                              } ,
	};
}

