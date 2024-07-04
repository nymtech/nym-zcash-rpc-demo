use anyhow::Result;
use tokio::io::AsyncReadExt;

// Reads from Read until the reader is exhausted and fills the send_buffer with the data read.
pub async fn fill_unbounded_buffer<T: AsyncReadExt + Unpin>(
    reader: &mut T,
    send_buffer: &mut Vec<u8>,
) -> Result<()> {
    loop {
        let mut buffer = vec![0; 100];
        let r = reader.read(&mut buffer).await?;
        println!("<< Received {} bytes from tcpstream", r);
        if r < buffer.len() {
            send_buffer.extend_from_slice(&buffer[..r]);
            break;
            // we're done and should send stuff
        } else {
            send_buffer.extend_from_slice(&buffer);
        }
    }
    Ok(())
}
