use clap::{Parser, Subcommand};

use std::path::PathBuf;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Subcommand, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Command {
    Decode {
        value: String,
    },
    Info {
        torrent: PathBuf,
    },
    Peers {
        torrent: PathBuf,
    },
    Handshake {
        torrent: PathBuf,
        peer: String,
    },

    DownloadPiece {
        #[arg(short)]
        output: PathBuf,
        torrent: PathBuf,
        piece: usize,
    },
    MagnetParse {
        link: String,
    },
    MagnetHandshake {
        link: String,
    },
    MagnetInfo {
        link: String,
    },
    Download {
        #[arg(short)]
        output: PathBuf,
        torrent: PathBuf,
    },
}
