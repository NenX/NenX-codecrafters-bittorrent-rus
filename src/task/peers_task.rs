use std::path::Path;

use anyhow::Context;

use crate::{
    my_impl::{MyTorrent, MyTorrentInfoKeys, MyTrackerPeers, MyTrackerRequest, MyTrackerResponse},
    MyTorrentResult,
};

pub async fn peers_task<T: AsRef<Path>>(torrent: T) -> MyTorrentResult<MyTrackerPeers> {
    let b = MyTorrent::from_file(torrent);
    let len = if let MyTorrentInfoKeys::SingleFile { length } = b.info.keys {
        length
    } else {
        todo!()
    };

    let request_params = MyTrackerRequest {
        // pubinfo_hash: hx,
        peer_id: String::from("00112233445566778899"),
        port: 6881,
        uploaded: 0,
        downloaded: 0,
        left: len,
        compact: 1,
    };
    let request_params = serde_urlencoded::to_string(&request_params).context("url encode")?;

    let request_params = format!(
        "{}?info_hash={}&{}",
        b.announce,
        b.info.urlencode(),
        request_params
    );
    println!("request_params {}", request_params);
    let res_bytes = reqwest::get(request_params).await?.bytes().await?;
    let res: MyTrackerResponse = serde_bencode::from_bytes(&res_bytes)?;
    res.peers.print();

    Ok(res.peers)
}
