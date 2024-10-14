use std::path::Path;

use crate::{
    my_impl::{MyConnect, MyTorrent},
    MyTorrentResult,
};

pub async fn handshake_task<T: AsRef<Path>>(torrent: T, peer: &str) -> MyTorrentResult<MyConnect> {
    let torrent = MyTorrent::from_file(torrent);
    let _ins = MyConnect::handshake(&torrent, peer).await;

    Ok(_ins)
}
