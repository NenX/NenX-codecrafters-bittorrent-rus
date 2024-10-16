use std::path::Path;

use anyhow::Result;

use crate::my_impl::{MyConnect, MyMagnet, MyTorrent};

pub async fn handshake_task<T: AsRef<Path>>(torrent: T, peer: &str) -> Result<MyConnect> {
    let torrent = MyTorrent::from_file(torrent);
    let _ins = MyConnect::handshake(&torrent, peer).await?;

    Ok(_ins)
}
pub async fn magnet_handshake_task(link: &str) -> Result<()> {
    let mag = MyMagnet::from_link(link)?;
    let mut conn = MyConnect::magnet_handshake(&mag).await?;

    let (mut peer_framed, payload) = conn.magnet_pre_download().await?;
    Ok(())
}
pub async fn magnet_parse_info(link: &str) -> Result<()> {
    let mag = MyMagnet::from_link(link)?;
    let _ins = MyConnect::magnet_info(&mag).await?;

    Ok(())
}
