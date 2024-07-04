use anyhow::Result;
use clap::Parser;
use nym_sdk::mixnet::{IncludedSurbs, MixnetClient, MixnetMessageSender, Recipient};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tokio_stream::StreamExt;
use utils::fill_unbounded_buffer;

mod utils;

#[derive(Parser, Debug)]
struct Args {
    /// Send timeout in seconds
    #[clap(long, default_value_t = 1)]
    send_timeout: u64,

    /// Receive timeout in seconds
    #[clap(long, default_value_t = 3)]
    receive_timeout: u64,

    /// Send and receive retry count
    #[clap(short, long, default_value_t = 3)]
    retry_count: u8,

    /// Mixnet server address
    #[clap(short, long)]
    server_address: String,

    /// Listen address
    #[clap(long, default_value = "127.0.0.1")]
    listen_address: String,

    /// Listen port
    #[clap(long, default_value = "8080")]
    listen_port: String,
}

lazy_static::lazy_static! {
    static ref ARGS: Args = Args::parse();
}

async fn handle_incoming(
    socket: TcpStream,
    server_addr: Recipient,
    client: Arc<RwLock<MixnetClient>>,
) -> Result<()> {
    let (mut read, mut write) = socket.into_split();

    let mut send_buffer = Vec::new();

    fill_unbounded_buffer(&mut read, &mut send_buffer).await?;
    let send_len = send_buffer.len();

    let recv = Arc::clone(&client);
    let sender = Arc::clone(&client);

    let recv_handle = tokio::spawn(async move {
        let mut retry = 0;
        loop {
            match timeout(
                Duration::from_secs(ARGS.receive_timeout),
                recv.write().await.next(),
            )
            .await
            {
                Ok(Some(received)) => match write.write_all(&received.message).await {
                    Ok(_) => {
                        println!(">> wrote to tcpstream successfully");
                        break;
                    }
                    Err(err) => {
                        retry += 1;
                        if retry > ARGS.retry_count {
                            println!("Failed to write to tcpstream after 3 retries");
                            break;
                        }
                        println!("write error: {err}");
                    }
                },
                Ok(None) => println!("No message received"),
                Err(e) => println!("Failed to receive message: {e}"),
            }
        }
    });

    let send_handle = tokio::spawn(async move {
        let mut retry = 0;
        loop {
            let mixnet_sender = sender.read().await.split_sender();
            match timeout(
                Duration::from_secs(ARGS.send_timeout),
                mixnet_sender.send_message(server_addr, &send_buffer, IncludedSurbs::Amount(100)),
            )
            .await
            {
                Ok(_) => {
                    println!(
                        ">> sent {} bytes to {} via the mixnet",
                        send_len, server_addr
                    );
                    break;
                }
                Err(e) => {
                    retry += 1;
                    if retry > ARGS.retry_count {
                        println!("Failed to send message after 3 retries");
                        break;
                    }
                    println!("Failed to send message: {e}")
                }
            };
        }
    });

    let results = futures::future::join_all(vec![send_handle, recv_handle]).await;
    for result in results {
        match result {
            Ok(_) => {}
            Err(e) => {
                print!("Failed to send/receive message: {e}");
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let server_addr = Recipient::from_str(&ARGS.server_address)?;
    // nym_bin_common::logging::setup_logging();

    println!(":: creating client...");
    let client = MixnetClient::connect_new().await?;
    let client_addr = &client.nym_address();
    println!(":: client created: {}", &client_addr);
    //     let mutex_client: Arc<Mutex<MixnetClient>> = Arc::new(Mutex::new(client));
    let rw_client: Arc<RwLock<MixnetClient>> = Arc::new(RwLock::new(client));

    let listener =
        TcpListener::bind(format!("{}:{}", ARGS.listen_address, ARGS.listen_port)).await?;
    println!(":: listening on {}", ARGS.listen_port);
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle_incoming(socket, server_addr, Arc::clone(&rw_client)));
    }
}
