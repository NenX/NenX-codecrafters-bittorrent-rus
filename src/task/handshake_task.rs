use std::path::Path;

use anyhow::Result;

use crate::my_impl::{MyConnect, MyMagnet, MyTorrent};

pub async fn handshake_task<T: AsRef<Path>>(torrent: T, peer: &str) -> Result<MyConnect> {
    let torrent = MyTorrent::from_file(torrent);
    let _ins = MyConnect::handshake(&torrent, peer).await?;

    Ok(_ins)
}
pub async fn magnet_handshake_task(link: &str) -> Result<MyConnect> {
    let mag = MyMagnet::from_link(link)?;
    let _ins = MyConnect::magnet_handshake(&mag).await?;

    Ok(_ins)
}
pub async fn magnet_parse_info(link: &str) -> Result<MyConnect> {
    let mag = MyMagnet::from_link(link)?;
    let _ins = MyConnect::magnet_handshake(&mag).await?;

    Ok(_ins)
}
