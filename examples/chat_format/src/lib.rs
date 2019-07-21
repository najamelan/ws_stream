pub mod futures_serde_cbor;


use
{
	serde :: { Serialize, Deserialize } ,
};


#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
//
pub enum Command
{
	Message       ,
	ServerMessage ,
	SetNick       ,
	Disconnect    ,
	Connect       ,
}



/// Wire format for communication between the server and clients
/// Clients only need to set cmd and txt if they just want to send a chat message.
//
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
//
pub struct ChatMessage
{
	pub cmd : Command        ,
	pub txt : Option<String> ,
	pub nick: Option<String> ,

	/// A unique sender id, even if the nick changes, this will be constant for the entire
	/// websocket connection. This allows clients to give a username a specific color and
	/// keep it if the nick changes.
	//
	pub sid : Option<usize> ,
}

