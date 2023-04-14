use anyhow::Result;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;

use crate::process::Process;

mod process;
mod protocol;
use protocol::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let s = tracing_subscriber::fmt().with_max_level(Level::TRACE).finish();
    tracing::subscriber::set_global_default(s)?;
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_connection(stream));
            }
            Err(e) => tracing::trace!("couldn't get client: {:?}", e),
        }
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    loop {
        stream.readable().await?;
        let mut buf = [0u8; 4096];
        match stream.try_read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                tracing::trace!(
                    "read {} bytes, {:?}",
                    n,
                    std::str::from_utf8(&buf[..n]).unwrap()
                );
                let p = Parser::new(&buf);
                tracing::trace!("parsing finished");
                let fs = p.process();
                tracing::trace!("processing finished");
                stream
                    .write_all(
                        &fs.into_iter()
                            .map(|x| x.into())
                            .collect::<Vec<Vec<u8>>>()
                            .concat(),
                    )
                    .await?;
            }
            Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    Ok(())
}
