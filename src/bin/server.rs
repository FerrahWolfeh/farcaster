use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};

use structopt::StructOpt;

use farcaster::{Protocol, DEFAULT_SERVER_ADDR};

#[derive(Debug, StructOpt)]
#[structopt(name = "server")]
struct Args {
    /// Service listening address
    #[structopt(long, default_value = DEFAULT_SERVER_ADDR, global = true)]
    addr: SocketAddr,
}

/// Given a TcpStream:
/// - Deserialize the request
/// - Handle the request
/// - Serialize and write the Response to the stream
fn handle_connection(stream: TcpStream) -> io::Result<()> {
    let mut protocol = Protocol::with_stream(stream)?;

    protocol.read_message()?;

    Ok(())
    // let resp = match request {
    //     Request::Echo(message) => Response(format!("'{}' from the other side!", message)),
    //     Request::Jumble { message, amount } => Response(jumble_message(&message, amount)),
    // };

    // protocol.send_message(&resp)
}

fn main() -> io::Result<()> {
    let args = Args::from_args();
    eprintln!("Starting server on '{}'", args.addr);

    let listener = TcpListener::bind(args.addr)?;
    for stream in listener.incoming().flatten() {
        std::thread::spawn(move || {
            handle_connection(stream).map_err(|e| eprintln!("Error: {}", e))
        });
    }
    Ok(())
}
