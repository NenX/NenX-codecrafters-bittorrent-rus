use std::collections::HashMap;

use crate::calc_target_chunk_length;

use super::{MyExtHandshakePayload, MyRequestPayload, MyTorrent};
const BLOCK_SIZE_MAX: usize = 1 << 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MyPeerMsgTag {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
    Extendsion = 20,
}
impl TryFrom<u8> for MyPeerMsgTag {
    type Error = std::io::Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let tag = match value {
            0 => Self::Choke,
            1 => Self::Unchoke,
            2 => Self::Interested,
            3 => Self::NotInterested,
            4 => Self::Have,
            5 => Self::Bitfield,
            6 => Self::Request,
            7 => Self::Piece,
            8 => Self::Cancel,
            _ => return Err(std::io::ErrorKind::InvalidData.into()),
        };
        Ok(tag)
    }
}
#[derive(Debug, Clone)]
pub struct MyPeerMsg {
    pub tag: MyPeerMsgTag,
    pub payload: Vec<u8>,
}

impl From<MyRequestPayload> for MyPeerMsg {
    fn from(value: MyRequestPayload) -> Self {
        let b = value.to_bytes();
        Self {
            tag: MyPeerMsgTag::Request,
            payload: b.to_vec(),
        }
    }
}
impl MyPeerMsg {
    pub fn ext_handshake() -> Self {
        let a = MyExtHandshakePayload::default();
        Self {
            tag: MyPeerMsgTag::Extendsion,
            payload: a.to_bytes().unwrap(),
        }
    }
    pub fn interested() -> Self {
        Self {
            tag: MyPeerMsgTag::Interested,
            payload: Vec::new(),
        }
    }
    pub fn request(index: u32, begin: u32, length: u32) -> Self {
        let request = MyRequestPayload::new(index, begin, length);
        Self {
            tag: MyPeerMsgTag::Request,
            payload: request.to_bytes().to_vec(),
        }
    }
    pub fn request_iter(piece_i: usize, b: &MyTorrent) -> impl Iterator<Item = Self> {
        let info = &b.info;
        let length = b.single_length().unwrap();
        let piece_size =
            calc_target_chunk_length(length, info.piece_length, info.pieces.0.len(), piece_i);

        let block_n = (piece_size + BLOCK_SIZE_MAX - 1) / BLOCK_SIZE_MAX;

        let it = 0..=block_n - 1;

        it.map(move |block_i| {
            let block_size = calc_target_chunk_length(piece_size, BLOCK_SIZE_MAX, block_n, block_i);

            Self::request(
                piece_i as u32,
                (block_i * BLOCK_SIZE_MAX) as u32,
                block_size as u32,
            )
        })
    }
}
