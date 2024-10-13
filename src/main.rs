use bittorrent_starter_rust::{
    commands::{Args, Command},
    decode_task, downloadpiece_task, handshake_task, info_task, peers_task, MyTorrentResult,
};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Decode { value } => {
            decode_task(&value).await?;
        }
        Command::Info { torrent } => {
            let _ = info_task(torrent);
        }
        Command::Peers { torrent } => {
            let _ = peers_task(torrent).await;
        }
        Command::Handshake { torrent, peer } => {
            let _ = handshake_task(torrent, &peer).await;
        }
        Command::DownloadPiece {
            output,
            torrent,
            piece,
        } => downloadpiece_task(torrent, output, piece).await?,
    }
    Ok(())
}

#[test]
fn ta() -> MyTorrentResult<()> {
    use bytes::{BufMut, BytesMut};

    let mut buf = BytesMut::with_capacity(128);
    buf.put(&[0; 64][..]);

    let ptr = buf.as_ptr();
    let other = buf.split();

    assert!(buf.is_empty());
    assert_eq!(buf.capacity(), 64);

    drop(other);
    buf.reserve(128);

    assert_eq!(buf.capacity(), 128);
    assert_eq!(buf.as_ptr(), ptr);
    Ok(())
}
