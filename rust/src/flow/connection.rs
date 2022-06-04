use super::frame::Frame;
use super::Result;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    reading_connect_packet: bool,
}

// TODO:  Figure out what this is set to on the C++ side.
const MAX_FDB_FRAME_LENGTH: usize = 1024 * 1024;

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(MAX_FDB_FRAME_LENGTH),
            reading_connect_packet: true,
        }
    }

    pub async fn send_connect_packet(&mut self) -> Result<()> {
        let connect_packet = super::frame::ConnectPacket::new();
        self.stream.write_all(&connect_packet.as_bytes()).await?;
        self.stream.flush().await?;
        Ok(())
    }

    // TODO: Pass this a lambda, and change the payload in frame from a vec<u8> to a &'a[u8].
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if self.reading_connect_packet {
                if let Some(_connect_packet) = super::frame::get_connect_packet(&mut self.buffer)? {
                    self.reading_connect_packet = false;
                    continue;
                }
            } else {
                if let Some(frame) = super::frame::get_frame(&mut self.buffer)? {
                    return Ok(Some(frame));
                }
            }
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // eof
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(format!(
                        "connection closed mid-frame {} bytes left: {:x?}",
                        self.buffer[..].len(),
                        &self.buffer[..]
                    )
                    .into());
                }
            }
        }
    }
    pub async fn write_frame(&mut self, frame: Frame) -> Result<()> {
        let buf = frame.as_bytes();
        self.stream.write_all(&buf).await?;
        self.stream.flush().await?;
        Ok(())
    }
}