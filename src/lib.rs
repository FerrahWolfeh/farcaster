//! Message Serialization/Deserialization (Protocol) for client <-> server communication
//!
//! Ideally you would use some existing Serialization/Deserialization,
//! but this is here to see what's going on under the hood.
//!

use serde::{Deserialize, Serialize};
use std::convert::From;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};

pub const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:4000";

/// The default memory layout for this cast is as follows:
/// ```ignore
/// |    u16     |     [u8]     |     u16    |     [u8]     |   i64   |
/// |   length   |   username   |   length   |   password   |  expiry |
/// ```
///
#[derive(Debug, Serialize, Deserialize)]
pub struct FireCmd {
    pub username: String,
    pub password: String,
    pub expiry: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FCPayload<S: Serialize> {
    pub payload: S,
}

/// Request object (client -> server)
#[derive(Debug)]
pub enum Request {
    /// Echo a message back
    Echo(String),
    /// Jumble up a message with given amount of entropy before echoing
    Jumble { message: String, amount: u16 },
}

/// Encode the Request type as a single byte (as long as we don't exceed 255 types)
///
/// We use `&Request` since we don't actually need to own or mutate the request fields
impl From<&Request> for u8 {
    fn from(req: &Request) -> Self {
        match req {
            Request::Echo(_) => 1,
            Request::Jumble { .. } => 2,
        }
    }
}

/// Message format for Request is:
/// ```ignore
/// |    u8    |     u16     |     [u8]      | ... u16    |   ... [u8]         |
/// |   type   |    length   |  value bytes  | ... length |   ... value bytes  |
/// ```
///
/// Starts with a type, and then is an arbitrary length of (length/bytes) tuples
impl Request {
    /// View the message portion of this request
    pub fn message(&self) -> &str {
        match self {
            Request::Echo(message) => message,
            Request::Jumble { message, .. } => message,
        }
    }
}

// impl Serialize for FireCmd {
//     fn serialize(&self, buf: &mut impl Write) -> io::Result<usize> {
//         let mut bytes_written: usize = 1;
//         let uname = self.username.as_bytes();
//         let pwd = self.password.as_bytes();

//         let u16bsize = size_of::<u16>();
//         let i64bsize = size_of::<i64>();

//         // write username field
//         buf.write_u16::<NetworkEndian>(uname.len() as u16)?;
//         buf.write_all(uname)?;
//         bytes_written += u16bsize + uname.len();

//         buf.write_u16::<NetworkEndian>(pwd.len() as u16)?;
//         buf.write_all(pwd)?;
//         bytes_written += u16bsize + pwd.len();

//         buf.write_i64::<NetworkEndian>(self.expiry)?;
//         bytes_written += i64bsize;

//         Ok(bytes_written)
//     }
// }

/// Abstracted Protocol that wraps a TcpStream and manages
/// sending & receiving of messages
pub struct Protocol {
    reader: io::BufReader<TcpStream>,
    stream: TcpStream,
}

impl Protocol {
    /// Wrap a TcpStream with Protocol
    pub fn with_stream(stream: TcpStream) -> io::Result<Self> {
        Ok(Self {
            reader: io::BufReader::new(stream.try_clone()?),
            stream,
        })
    }

    /// Establish a connection, wrap stream in BufReader/Writer
    pub fn connect(dest: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(dest)?;
        eprintln!("Connecting to {}", dest);
        Self::with_stream(stream)
    }

    /// Serialize a message to the server and write it to the TcpStream
    pub fn send_message(&mut self, message: &impl Serialize) -> io::Result<()> {
        let bdata = bincode::serialize(message).unwrap();

        self.stream.write_all(&bdata)?;
        self.stream.flush()
    }

    /// Read a message from the inner TcpStream
    ///
    /// NOTE: Will block until there's data to read (or deserialize fails with io::ErrorKind::Interrupted)
    ///       so only use when a message is expected to arrive
    #[allow(clippy::unused_io_amount)]
    pub fn read_message(&mut self) -> Result<(), io::Error> {
        let mut outbuffer = vec![0; self.reader.capacity()];
        self.reader.read(&mut outbuffer)?;

        eprintln!(
            "Incoming ersc encoded [{}] [{:?}]",
            String::from_utf8_lossy(&outbuffer),
            self.stream.peer_addr()
        );

        Ok(())
    }
}
