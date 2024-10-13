use futures_util::{SinkExt, StreamExt};
use std::path::Path;

use anyhow::Context;
use sha1::{Digest, Sha1};

use crate::{
    handshake_task,
    my_impl::{MyConnect, MyPeerMsg, MyPeerMsgFramed, MyPeerMsgTag, MyPiece, MyRequest, MyTorrent},
    peers_task, MyTorrentResult,
};
const BLOCK_MAX: usize = 1 << 14;
pub async fn downloadpiece_task<T: AsRef<Path>>(
    torrent: T,
    output: T,
    piece: usize,
) -> MyTorrentResult<()> {
    let a = MyConnect::downlaod_piece_at(torrent, output, piece).await;
    a
}
pub async fn downloadpiece_task_old<T: AsRef<Path>>(
    torrent: T,
    output: T,
    piece: usize,
) -> MyTorrentResult<()> {
    let piece_i = piece;
    println!("downloadpiece_task");
    let t = MyTorrent::from_file(&torrent);
    let peers = peers_task(&torrent).await?;
    let first_one = &peers.0.get(0).unwrap().to_string();
    let mut c = handshake_task(torrent, first_one).await?;
    let peer = &mut c.remote_socket;
    let length = t.single_length().unwrap();
    // ----------

    let mut peer_framed = tokio_util::codec::Framed::new(peer, MyPeerMsgFramed);

    let bitfield = peer_framed
        .next()
        .await
        .expect("peer always sends a bitfields")
        .context("peer message was invalid")?;
    assert_eq!(bitfield.tag, MyPeerMsgTag::Bitfield);
    // NOTE: we assume that the bitfield covers all pieces
    peer_framed
        .send(MyPeerMsg {
            tag: MyPeerMsgTag::Interested,
            payload: Vec::new(),
        })
        .await
        .context("send interested message")?;
    let unchoke = peer_framed
        .next()
        .await
        .expect("peer always sends an unchoke")
        .context("peer message was invalid")?;
    assert_eq!(unchoke.tag, MyPeerMsgTag::Unchoke);
    assert!(unchoke.payload.is_empty());
    let piece_hash = &t.info.pieces.0[piece_i];
    let piece_size = if piece_i == t.info.pieces.0.len() - 1 {
        let md = length % t.info.piece_length;
        if md == 0 {
            t.info.piece_length
        } else {
            md
        }
    } else {
        t.info.piece_length
    };
    // the + (BLOCK_MAX - 1) rounds up
    let nblocks = (piece_size + (BLOCK_MAX - 1)) / BLOCK_MAX;
    let mut all_blocks = Vec::with_capacity(piece_size);
    for block in 0..nblocks {
        let block_size = if block == nblocks - 1 {
            let md = piece_size % BLOCK_MAX;
            if md == 0 {
                BLOCK_MAX
            } else {
                md
            }
        } else {
            BLOCK_MAX
        };
        let mut request = MyRequest::new(
            piece_i as u32,
            (block * BLOCK_MAX) as u32,
            block_size as u32,
        );
        let request_bytes = Vec::from(request.as_bytes());
        peer_framed
            .send(MyPeerMsg {
                tag: MyPeerMsgTag::Request,
                payload: request_bytes,
            })
            .await
            .with_context(|| format!("send request for block {block}"))?;
        let piece = peer_framed
            .next()
            .await
            .expect("peer always sends a piece")
            .context("peer message was invalid")?;
        assert_eq!(piece.tag, MyPeerMsgTag::Piece);
        assert!(!piece.payload.is_empty());
        let piece = MyPiece::ref_from_bytes(&piece.payload[..])
            .expect("always get all Piece response fields from peer");
        assert_eq!(piece.index() as usize, piece_i);
        assert_eq!(piece.begin() as usize, block * BLOCK_MAX);
        assert_eq!(piece.block().len(), block_size);
        all_blocks.extend(piece.block());
    }
    assert_eq!(all_blocks.len(), piece_size);
    let mut hasher = Sha1::new();
    hasher.update(&all_blocks);
    let hash: [u8; 20] = hasher
        .finalize()
        .try_into()
        .expect("GenericArray<_, 20> == [_; 20]");
    assert_eq!(&hash, piece_hash);
    tokio::fs::write(&output, all_blocks)
        .await
        .context("write out downloaded piece")?;
    println!(
        "Piece {piece_i} downloaded to {}.",
        output.as_ref().display()
    );
    Ok(())
}

#[tokio::test]
async fn f() {
    downloadpiece_task("sample.torrent", "./aa", 0)
        .await
        .expect("hh");
}
