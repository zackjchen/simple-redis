use crate::{
    backend::Backend,
    cmd::{Command, CommandExecuter},
    resp::{frame::RespFrame, simple_error::SimpleError, RespDecode, RespEncode, RespError},
};
use anyhow::Result;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::{info, warn};

#[derive(Debug)]
struct RedisRequest {
    frame: RespFrame,
    backend: Backend,
}
struct RedisResponse {
    frame: RespFrame,
}

#[derive(Debug)]
struct RespFrameCodec;

/// how to get a frame from a stream
/// call request_handler with the frame
/// send the response back to the client
// The backend here is Arc<BackendInner>
pub async fn stream_handler(stream: TcpStream, backend: Backend) -> Result<()> {
    let mut framed = Framed::new(stream, RespFrameCodec);
    loop {
        match framed.next().await {
            Some(Ok(frame)) => {
                info!("Received frame: {:?}", frame);
                let request = RedisRequest {
                    frame,
                    backend: backend.clone(),
                };
                let response = match request_handler(request).await {
                    Ok(response) => response,
                    Err(e) => RedisResponse {
                        frame: SimpleError::new(e.to_string()).into(),
                    },
                };
                info!("Sending response: {:?}", response.frame);
                framed.send(response.frame).await?;
            }
            Some(Err(e)) => {
                warn!("Error decoding frame: {}", e);
                framed
                    .send(RespFrame::SimpleError(SimpleError::new(e.to_string())))
                    .await?
            }
            None => {
                return {
                    info!("Connection closed");
                    Ok(())
                }
            }
        };
    }
}

async fn request_handler(request: RedisRequest) -> Result<RedisResponse> {
    let (frame, backend) = (request.frame, request.backend);
    let command = Command::try_from(frame)?;
    info!("Executing command: {:?}", command);
    let response = command.execute(backend);
    Ok(RedisResponse { frame: response })
}

impl Encoder<RespFrame> for RespFrameCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RespFrame, dst: &mut bytes::BytesMut) -> Result<()> {
        let encoded = item.encode();
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}

impl Decoder for RespFrameCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>> {
        match RespFrame::decode(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(RespError::NotCompleteFrame) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
