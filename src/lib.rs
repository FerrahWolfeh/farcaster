//! # Encrypted message passing protocol FarCaster.
//!
//! This utility helps passing arbitrary data over a TCP connection with the option of it being encrypted or not.
//!
//! Useful for passing basic commands or complex bytecode to and from any device that has a network connection.
//!
//!

use log::{debug, trace};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use std::net::{SocketAddr, TcpStream};

pub mod error;

pub trait Payload {
    type Struct;

    fn encrypt(self, key: &[u8]) -> Vec<u8>;
    fn decrypt(encoded: Vec<u8>, key: &[u8]) -> Self::Struct;
    fn decode(encoded: Vec<u8>) -> Self::Struct;
    fn encode(self) -> Vec<u8>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FCPayload<S: std::fmt::Debug + Payload + Serialize> {
    pub descriptor: u8,
    payload: S,
}

impl<S: std::fmt::Debug + Payload + Serialize> FCPayload<S> {
    pub fn _raw(&self) -> Vec<u8> {
        Vec::new()
    }

    pub fn payload_encrypted(&self, _key: &[u8]) -> Vec<u8> {
        Vec::new()
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
pub struct CannonLauncher<S: std::fmt::Debug + Payload + Serialize> {
    reader: io::BufReader<TcpStream>,
    stream: TcpStream,
    payload: Option<FCPayload<S>>,
}

impl<S: std::fmt::Debug + Payload + Serialize> CannonLauncher<S> {
    /// Wrap a TcpStream with Protocol
    pub fn with_stream(stream: TcpStream) -> io::Result<Self> {
        Ok(Self {
            reader: io::BufReader::new(stream.try_clone()?),
            stream,
            payload: None,
        })
    }

    pub fn set_payload(&mut self, payload: FCPayload<S>) -> &mut Self {
        self.payload = Some(payload);

        trace!("Current payload in [{:p}]: {:?}", &self, self.payload);

        self
    }

    pub fn clear_payload(&mut self) {
        self.payload = None;
        trace!("Cleared payload in [{:p}]", &self);
    }

    /// Establish a connection, wrap stream in BufReader/Writer
    pub fn connect(dest: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(dest)?;
        debug!("Connecting to {}", dest);
        Self::with_stream(stream)
    }

    /// Serialize a message to the server and write it to the TcpStream
    pub fn send(&mut self) -> io::Result<()> {
        let bdata = bincode::serialize(&self.payload).unwrap();

        self.stream.write_all(&bdata)?;
        self.stream.flush()
    }

    /// Read a message from the inner TcpStream
    ///
    /// NOTE: Will block until there's data to read (or deserialize fails with io::ErrorKind::Interrupted)
    ///       so only use when a message is expected to arrive
    #[allow(clippy::unused_io_amount)]
    pub fn read_message(&mut self) -> Result<(), io::Error> {
        let outbuffer = self.reader.fill_buf()?;

        let _abuf = outbuffer.to_vec();

        Ok(())
    }
}
