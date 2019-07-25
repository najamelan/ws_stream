pub mod futures_serde_cbor;


use
{
	serde :: { Serialize, Deserialize } ,
};


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
	NickChanged { old  : String, sid: usize, new: String  } ,
	Welcome     { users: Vec<(usize,String)>, txt: String } ,
}

