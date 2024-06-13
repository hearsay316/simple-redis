use std::io;
use std::net::SocketAddr;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tracing::{info, warn};

const BUF_SIZE: usize = 4096;

#[tokio::main]
async fn main() -> Result<()> {
    info!("Hello, world!");
    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:6379";
    let listener = TcpListener::bind(addr).await?;
    println!("dredis链接地址{}",addr);
    loop {
        let (socket, raddr) = listener.accept().await?;
        info!( "accepted connection{}",raddr);
        tokio::spawn(async move {
         if let Err(e) =   process_redis_conn(socket,raddr).await{
             warn!("error processing connection; error = {:?}",e);
         }
        });
    }
}

async fn process_redis_conn(mut stream: tokio::net::TcpStream,raddr:SocketAddr) -> Result<()> {
    loop {
        stream.readable().await?;
        let mut buf = Vec::with_capacity(BUF_SIZE);
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                info!("read {} bytes",n);
                let line =String::from_utf8_lossy(&buf);
                info!("read line:{:?}",line);
                stream.write_all(b"+OK\r\n").await?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            },
            Err(e)=>{
                return Err(e.into());
            }
        }
    }
    warn!("closing connection from {}",raddr);
    Ok(())
}