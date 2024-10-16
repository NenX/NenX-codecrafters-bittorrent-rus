use std::path::Path;

use anyhow::Result;

use crate::{
    my_impl::{MyConnect, MyMagnet, MyTorrent},
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

pub async fn download_task<T: AsRef<Path>>(torrent: T, output: T) -> MyTorrentResult<()> {
    let torrent = MyTorrent::from_file(torrent);

    MyConnect::downlaod_all(&torrent, output).await
}

pub async fn magnet_download_piece_task<T: AsRef<Path>>(
    link: &str,
    output: T,
    piece: usize,
) -> Result<()> {
    MyConnect::magnet_downlaod_piece_at(&MyMagnet::from_link(link).unwrap(), output, piece).await?;

    Ok(())
}
pub async fn magnet_download_task<T: AsRef<Path>>(link: &str, output: T) -> Result<()> {
    MyConnect::magnet_downlaod(&MyMagnet::from_link(link).unwrap(), output).await?;

    Ok(())
}

#[tokio::test]
async fn f() {
    downloadpiece_task("sample.torrent", "./aa", 0)
        .await
        .expect("hh");
}
