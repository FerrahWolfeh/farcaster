use std::io;
use std::net::SocketAddr;

use structopt::StructOpt;

use farcaster::{FCPayload, FireCmd, Protocol, DEFAULT_SERVER_ADDR};

#[derive(Debug, StructOpt)]
#[structopt(name = "client")]
struct Args {
    #[structopt(long, default_value = DEFAULT_SERVER_ADDR, global = true)]
    addr: SocketAddr,
}

fn main() -> io::Result<()> {
    let args = Args::from_args();

    let req = FireCmd {
        username: "juninho@999".to_string(),
        password: "24afsa!@%T%%Aasfa".to_string(),
        expiry: -167888213,
    };

    let payload = FCPayload { payload: req };

    Protocol::connect(args.addr)
        .and_then(|mut client| {
            client.send_message(&payload)?;
            Ok(client)
        })
        .unwrap();
    Ok(())
}
