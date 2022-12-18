//! # Encrypted message passing protocol FarCaster.
//!
//! This utility helps passing arbitrary data over a TCP connection with the option of it being encrypted or not.
//!
//! Useful for passing basic commands or complex bytecode to and from any device that has a network connection.
//!
//!

use aes_gcm::aead::consts::U12;
use aes_gcm::aead::Aead;
use aes_gcm::aes::Aes256;
use aes_gcm::{AesGcm, KeyInit, Nonce};
use log::{debug, trace};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use std::net::{SocketAddr, TcpStream};

pub mod error;

#[derive(Debug, Serialize, Deserialize)]
pub struct FCPayload {
    pub descriptor: u8,
    payload: Vec<u8>,
    metadata: Vec<u8>,
}

impl FCPayload {
    pub fn override_descriptor(&mut self, byte: u8) {
        self.descriptor = byte;
    }

    pub fn insert_raw_payload<P>(&mut self, payload: P) -> &mut Self
    where
        P: std::fmt::Debug + Serialize + DeserializeOwned,
    {
        debug!("Inserted payload: {payload:?}");

        self.payload = bincode::serialize(&payload).unwrap();

        trace!("Serialized payload data: {:X?}", &self.payload);
        self
    }

    pub fn insert_metadata<P>(&mut self, meta: P) -> &mut Self
    where
        P: std::fmt::Debug + Serialize + DeserializeOwned,
    {
        debug!("Inserted payload: {meta:?}");

        self.metadata = bincode::serialize(&meta).unwrap();

        trace!("Serialized payload data: {:X?}", &self.metadata);
        self
    }

    pub fn decode_metadata<P: DeserializeOwned + Sized>(&self) -> P {
        let deserialized: P = bincode::deserialize(&self.metadata).unwrap();
        deserialized
    }

    pub fn encrypt_payload(&mut self, key: &[u8; 256], nonce: &[u8; 96]) {
        let cipher: AesGcm<Aes256, U12> = AesGcm::new_from_slice(key).unwrap();

        let key_nonce = Nonce::from_slice(nonce); // 96-bits; Usually both servers should already know the same key at this point.

        let encrypted_load = cipher.encrypt(key_nonce, self.payload.as_ref()).unwrap();

        self.payload = encrypted_load;

        trace!("Encrypted payload data: {:X?}", self.payload);

        if cfg!(debug_assertions) {
            let test_decode = cipher.decrypt(key_nonce, self.payload.as_ref()).unwrap();
            assert_eq!(&test_decode, &self.payload);
        }
    }

    pub fn decode_raw_payload<P: DeserializeOwned + Sized>(&self) -> P {
        let deserialized: P = bincode::deserialize(&self.payload).unwrap();
        deserialized
    }

    pub fn decode_encrypted_payload<P: DeserializeOwned + Sized>(
        &self,
        key: &[u8; 256],
        nonce: &[u8; 96],
    ) -> P {
        let cipher: AesGcm<Aes256, U12> = AesGcm::new_from_slice(key).unwrap();

        let key_nonce = Nonce::from_slice(nonce); // 96-bits; Same thing as up there.

        let decoded = cipher.decrypt(key_nonce, self.payload.as_ref()).unwrap();

        let deserialized: P = bincode::deserialize(&decoded).unwrap();
        deserialized
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

/// Abstracted protocol that wraps a TcpStream and manages
/// sending & receiving of messages
pub struct CannonLauncher {
    reader: io::BufReader<TcpStream>,
    stream: TcpStream,
    payload: Option<FCPayload>,
}

impl CannonLauncher {
    /// Wrap a TcpStream with Protocol
    pub fn with_stream(stream: TcpStream) -> io::Result<Self> {
        Ok(Self {
            reader: io::BufReader::new(stream.try_clone()?),
            stream,
            payload: None,
        })
    }

    pub fn set_payload(&mut self, payload: FCPayload) -> &mut Self {
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
    pub fn read_message(&mut self) -> Result<FCPayload, io::Error> {
        let outbuffer = self.reader.fill_buf()?;

        let decoded_pl: FCPayload = bincode::deserialize(outbuffer).unwrap();

        Ok(decoded_pl)
    }
}
