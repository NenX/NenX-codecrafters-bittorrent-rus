use std::path::Path;

use crate::{
    my_impl::{MyTorrent, MyTrackerPeers},
    MyTorrentResult,
};

pub async fn peers_task<T: AsRef<Path>>(torrent: T) -> MyTorrentResult<MyTrackerPeers> {
    let b = MyTorrent::from_file(torrent);

    b.fetch_peers().await
}
