use std::path::Path;


use crate::{
    my_impl::{
        MyConnect, MyTorrent, MyTrackerPeers,
    },
    MyTorrentResult,
};

pub async fn peers_task<T: AsRef<Path>>(torrent: T) -> MyTorrentResult<MyTrackerPeers> {
    let b = MyTorrent::from_file(torrent);
    

    MyConnect::fetch_peers(&b).await
}
