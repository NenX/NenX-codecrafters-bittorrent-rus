use std::path::Path;

use anyhow::Context;

use crate::{
    my_impl::{
        MyConnect, MyTorrent, MyTorrentInfoKeys, MyTrackerPeers, MyTrackerRequest,
        MyTrackerResponse,
    },
    MyTorrentResult,
};

pub async fn peers_task<T: AsRef<Path>>(torrent: T) -> MyTorrentResult<MyTrackerPeers> {
    let b = MyTorrent::from_file(torrent);
    let a = MyConnect::fetch_peers(&b).await;

    a
}
