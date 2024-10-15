use std::path::Path;

use crate::{
    my_impl::{MyConnect, MyTorrent},
    MyTorrentResult,
};
pub async fn downloadpiece_task<T: AsRef<Path>>(
    torrent: T,
    output: T,
    piece: usize,
) -> MyTorrentResult<()> {
    let torrent = MyTorrent::from_file(torrent);

    
    MyConnect::downlaod_piece_at(&torrent, output, piece).await
}

pub async fn download_task<T: AsRef<Path>>(
    torrent: T,
    output: T,

) -> MyTorrentResult<()> {
    let torrent = MyTorrent::from_file(torrent);

    
    MyConnect::downlaod_all(&torrent, output).await
}

#[tokio::test]
async fn f() {
    downloadpiece_task("sample.torrent", "./aa", 0)
        .await
        .expect("hh");
}
