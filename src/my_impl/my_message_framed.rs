use std::io;

use anyhow::Context;
use anyhow::Result;
use bytes::Buf;
use bytes::BufMut;
use bytes::BytesMut;
use tokio_util::codec::Decoder;
use tokio_util::codec::Encoder;

use super::MyPeerMsg;
use super::MyPeerMsgTag;

const LEN_BYTE: usize = u32::BITS as usize / 8;
const MAX: usize = u16::MAX as usize;

pub struct MyPeerMsgFramed;

impl Decoder for MyPeerMsgFramed {
    type Item = MyPeerMsg;
    type Error = std::io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let total_len = src.len();

        if total_len < LEN_BYTE {
            return Ok(None);
        }
        let len_slice = src[..4].try_into().context("bytes to len").unwrap();
        let len = u32::from_be_bytes(len_slice) as usize;

        if len == 0 {
            src.advance(LEN_BYTE);
            return self.decode(src);
        }
        let expected_len = LEN_BYTE + len;
        if total_len < expected_len {
            src.reserve(expected_len - total_len);
            return Ok(None);
        }
        if len - 1 > MAX {
            return Err(io::ErrorKind::InvalidData.into());
        }

        let tag = MyPeerMsgTag::try_from(src[LEN_BYTE])
            .context("into tag")
            .unwrap();

        let payload = if len == 1 {
            vec![]
        } else {
            src[LEN_BYTE + 1..LEN_BYTE + len].to_vec()
        };

        Ok(Some(MyPeerMsg { payload, tag }))
    }
}
impl Encoder<MyPeerMsg> for MyPeerMsgFramed {
    type Error = std::io::Error;
    fn encode(&mut self, item: MyPeerMsg, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let payload_len = item.payload.len();

        if payload_len > MAX {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
        let total_len_slice = u32::to_be_bytes((payload_len + 1).try_into().unwrap());
        dst.reserve(LEN_BYTE + 1 + payload_len);

        dst.extend_from_slice(&total_len_slice);

        dst.put_u8(item.tag as u8);

        dst.extend_from_slice(&item.payload);
        println!("encode : {:?} {:?}", item, dst);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anyhow::{Context, Result};
    use futures_util::{SinkExt, StreamExt};
    use std::time::Duration;
    use tokio::time::{self};
    use tokio_util::codec;

    use super::*;

    #[tokio::test]
    async fn test1() -> Result<()> {
        let f1 = move || async move {
            let l = tokio::net::TcpListener::bind("0.0.0.0:2233").await.unwrap();
            let (socket, _) = l.accept().await.unwrap();
            let mut frame = codec::Framed::new(socket, MyPeerMsgFramed);

            let a = frame.next().await.context("read message").unwrap().unwrap();
            println!("f1 read ==> {:?}", a.tag);

            let m = MyPeerMsg {
                tag: MyPeerMsgTag::Interested,
                payload: vec![1].repeat(MAX),
            };
            frame.send(m.clone()).await.context("send").unwrap();
            frame.send(m).await.context("send").unwrap();
        };
        let f2 = || async {
            let mut socket = tokio::net::TcpStream::connect("0.0.0.0:2233")
                .await
                .unwrap();
            let mut frame = codec::Framed::new(socket, MyPeerMsgFramed);
            let m = MyPeerMsg {
                tag: MyPeerMsgTag::Interested,
                payload: vec![1, 2, u8::MAX, 4],
            };
            frame.send(m).await.context("send").unwrap();
            let a = frame.next().await.context("read message").unwrap().unwrap();
            let len = a.payload.len();
            let slice = a.payload.split_at(len - 5);
            println!("f2 read ==> {:?} {:?}", slice.1, a.tag);

            let a = frame.next().await.context("read message").unwrap().unwrap();
            println!("f2 read ==> {:?}", a.tag)
        };
        let h1 = tokio::spawn(async move {
            f1().await;
        });
        let h2 = tokio::spawn(async move {
            time::sleep(Duration::from_secs(1)).await;
            f2().await;
        });
        println!("??");
        h1.await.context("h1 wait").unwrap();
        h2.await.context("h2 wait").unwrap();
        Ok(())
    }
}
