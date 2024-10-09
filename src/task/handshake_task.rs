use anyhow::Context;
use serde_bencode::value::Value;

use crate::{
    my_impl::{MyConnect, MyTorrent, MyTorrentInfoKeys, MyTrackerRequest, MyTrackerResponse},
    torrent, MyBEncodedBuf, MyTorrentResult, Torrent, TrackerRequest, TrackerResponse,
};

pub async fn handshake_task(torrent: &str, peer: &str) -> MyTorrentResult<()> {
    let _ins = MyConnect::handshake(torrent, peer).await;

    Ok(())
}
