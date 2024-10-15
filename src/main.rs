use bittorrent_starter_rust::{
    commands::{Args, Command},
    decode_task, download_task, downloadpiece_task, handshake_task, info_task, magnet_parse_task,
    peers_task,
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
        Command::Download { output, torrent } => download_task(torrent, output).await?,
        Command::MagnetParse { link } => magnet_parse_task(&link)?,
    }
    Ok(())
}


