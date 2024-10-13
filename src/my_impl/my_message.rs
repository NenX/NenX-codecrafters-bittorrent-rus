use bytes::BufMut;

use crate::calc_target_chunk_length;

use super::MyTorrent;
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
        let target = info.pieces.0.get(piece_i);
        let piece_size =
            calc_target_chunk_length(length, info.piece_length, info.pieces.0.len(), piece_i);

        let block_n = (piece_size + BLOCK_SIZE_MAX - 1) / BLOCK_SIZE_MAX;

        let it = 0..=block_n;

        let m = it.map(move |block_i| {
            let block_size = calc_target_chunk_length(piece_size, BLOCK_SIZE_MAX, block_n, block_i);

            return MyPeerMsg::request(
                piece_i as u32,
                (block_i * BLOCK_SIZE_MAX) as u32,
                block_size as u32,
            );
        });
        m
    }
}
#[derive(Debug)]
#[repr(C)]
pub struct MyRequestPayload {
    index: [u8; 4],
    begin: [u8; 4],
    length: [u8; 4],
}
impl MyRequestPayload {
    pub fn new(index: u32, begin: u32, length: u32) -> Self {
        Self {
            index: index.to_be_bytes(),
            begin: begin.to_be_bytes(),
            length: length.to_be_bytes(),
        }
    }
    pub fn to_bytes(&self) -> &[u8] {
        let a = self as *const Self as *const [u8; std::mem::size_of::<Self>()];
        let a = unsafe { &*a };
        a
    }
    pub fn ref_from_bytes(data: &[u8]) -> Option<&Self> {
        if data.len() < std::mem::size_of::<Self>() {
            return None;
        }
        let a = data as *const [u8] as *const Self;
        let a = unsafe { &*a };
        return Some(a);
    }
}
#[derive(Debug)]
#[repr(C)]
pub struct MyPiecePayload<T: ?Sized = [u8]> {
    index: [u8; 4],
    begin: [u8; 4],
    block: T,
}
impl MyPiecePayload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let a = self as *const Self as *const [u8; Self::PIECE_SIZE];
        let a = unsafe { &*a };
        let v: Vec<_> = a.iter().chain(self.block.iter()).cloned().collect();
        v
    }
    const PIECE_SIZE: usize = std::mem::size_of::<MyPiecePayload<()>>();
    pub fn ref_from_bytes(data: &[u8]) -> Option<&Self> {
        dbg!(data.len());
        dbg!(Self::PIECE_SIZE);
        if data.len() < Self::PIECE_SIZE {
            return None;
        }
        let correct_len = data.len() - Self::PIECE_SIZE;
        let fat_pointer_with_correct_len = &data[..correct_len] as *const [u8] as *const Self;
        let a = unsafe { &*fat_pointer_with_correct_len };
        return Some(a);
    }
}
#[test]
fn test() {}
