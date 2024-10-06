// use hashes::Hashes;
use serde::Deserialize;

/// A Metainfo file (also known as .torrent files).
#[derive(Debug, Clone, Deserialize)]
pub struct Torrent {
    /// The URL of the tracker.
    announce: String,
    info: Info,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Keys {
    /// If `length` is present then the download represents a single file.
    SingleFile {
        /// The length of the file in bytes.
        length: usize,
    },
    /// Otherwise it represents a set of files which go in a directory structure.
    ///
    /// For the purposes of the other keys in `Info`, the multi-file case is treated as only having
    /// a single file by concatenating the files in the order they appear in the files list.
    MultiFile { files: Vec<File> },
}
#[derive(Debug, Clone, Deserialize)]
pub struct Info {
    /// The suggested name to save the file (or directory) as. It is purely advisory.
    ///
    /// In the single file case, the name key is the name of a file, in the muliple file case, it's
    /// the name of a directory.
    name: String,
    /// The number of bytes in each piece the file is split into.
    ///
    /// For the purposes of transfer, files are split into fixed-size pieces which are all the same
    /// length except for possibly the last one which may be truncated. piece length is almost
    /// always a power of two, most commonly 2^18 = 256K (BitTorrent prior to version 3.2 uses 2
    /// 20 = 1 M as default).
    #[serde(rename = "piece length")]
    plength: usize,
    /// Each entry of `pieces` is the SHA1 hash of the piece at the corresponding index.
    // pieces: Hashes,
    #[serde(flatten)]
    keys: Keys,
}
#[derive(Debug, Clone, Deserialize)]
pub struct File {
    /// The length of the file, in bytes.
    length: usize,
    /// Subdirectory names for this file, the last of which is the actual file name
    /// (a zero length list is an error case).
    path: Vec<String>,
}