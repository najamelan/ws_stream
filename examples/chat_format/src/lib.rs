pub mod futures_serde_cbor;


use
{
	serde :: { Serialize, Deserialize } ,
};


/// Wire format for communication between the server and clients
/// Currently it's not possible with futures-codec to frame the AsyncRead
/// with a different type as the AsyncWrite, so we wrap our 2 types in
/// an enum.
//
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
//
pub enum Wire
{
	/// Messages coming from a client
	//
	Client(ClientMsg),

	/// Messages coming from the server
	//
	Server(ServerMsg),
}


/// Wire format for communication between the server and clients
//
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
//
pub enum ClientMsg
{
	ChatMsg(String),
	SetNick(String),
}


/// Wire format for communication between the server and clients
//
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
//
pub enum ServerMsg
{
	ServerMsg   (String)                                    ,
	ChatMsg     { nick : String, sid: usize, txt: String  } ,
	UserJoined  { nick : String, sid: usize               } ,
	UserLeft    { nick : String, sid: usize               } ,
	NickChanged { old  : String, new: String              } ,
	Welcome     { users: Vec<(usize,String)>, txt: String } ,
}

