use anyhow::Result;
use clap::Parser;
use nym_sdk::mixnet::{
    MixnetClientBuilder, MixnetClientSender, MixnetMessageSender, ReconstructedMessage,
    StoragePaths,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use utils::fill_unbounded_buffer;

mod utils;

#[derive(Parser, Debug)]
struct Args {
    /// Upstream address, ie lightwalletd address
    #[clap(short, long)]
    upstream_address: String,

    /// Config directory
    #[clap(short, long, default_value = "/tmp/mixnet-client")]
    config_dir: String,
}

lazy_static::lazy_static! {
    static ref ARGS: Args = Args::parse();
}

#[tokio::main]
async fn main() -> Result<()> {
    //    nym_bin_common::logging::setup_logging();
    println!(":: creating client...");
    let config_dir = PathBuf::from(&ARGS.config_dir);
    let storage_paths = StoragePaths::new_from_dir(&config_dir)?;
    let client = MixnetClientBuilder::new_with_default_storage(storage_paths)
        .await?
        .build()?;

    let mut client = client.connect_to_mixnet().await?;

    let client_addr = client.nym_address();
    let sender = Arc::new(RwLock::new(client.split_sender()));
    println!(":: client created: {}", &client_addr);

    while let Some(new_message) = client.next().await {
        //        println!("<< received {:?} from mixnet", &new_message.message);
        tokio::spawn(handle_mixnet_message(new_message, Arc::clone(&sender)));
    }

    tokio::signal::ctrl_c().await?;
    Ok(())
}

async fn handle_mixnet_message(
    message: ReconstructedMessage,
    sender: Arc<RwLock<MixnetClientSender>>,
) -> Result<()> {
    let surb = message.sender_tag.expect("no sender tag");
    println!("<< incoming message from sender_tag: {:?}", surb);

    println!(
        ">> writing {} bytes to zcashd node at {}",
        &message.message.len(),
        ARGS.upstream_address
    );
    let mut stream = TcpStream::connect(&*ARGS.upstream_address).await?;
    stream.write_all(&message.message).await?;
    println!(">> sent {} bytes successfully", message.message.len(),);

    let mut send_buffer = Vec::new();

    fill_unbounded_buffer(&mut stream, &mut send_buffer).await?;

    println!(
        ">> sending {} bytes as reply to {} via the mixnet",
        send_buffer.len(),
        surb.clone()
    );
    sender.write().await.send_reply(surb, send_buffer).await?;
    Ok(())
}
