
use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{info, warn};
use simple_redis::{network, Backend};

#[tokio::main]
async fn main()->Result<()> {
    tracing_subscriber::fmt::init();
    info!("运行开始啦 ");
    let addr = "0.0.0.0:6379";
    println!("开始了");
    info!("Simple-Redis-server is listening on {}",addr);
    let listener = TcpListener::bind(addr).await?;
    let backend = Backend::new();
    loop {
        let (stream,raddr) = listener.accept().await?;
        let clone_backend = backend.clone();
        info!("Accepted connection from :{} ",raddr);
        tokio::spawn(async move {
            match network::stream_handler(stream,clone_backend).await{
                Ok(_)=>{
                    info!("Commection from {} is handled successfully",raddr);
                }
                Err(e)=>warn!("handle error for  {}:{:?}",raddr,e)
            }
        });
    }


}
