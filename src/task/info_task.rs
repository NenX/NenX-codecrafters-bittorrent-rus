use std::path::Path;

use sha1::{Digest, Sha1};

use crate::{
    dict_get, dict_get_as, display_value,
    my_impl::MyTorrent,
    pieces_hash, value_as_bytes, value_as_int, MyBEncodedBuf, MyTorrentResult,
};

// Usage: 70edcac2611a8829ebf467a6849f5d8408d9d8f4
#[allow(dead_code)]
pub fn info_raw(file_name: &str) -> MyTorrentResult<()> {
    let a = std::fs::read(file_name)?;
    let mut buf = MyBEncodedBuf {
        pos: 0,
        inner_buf: a,
        outer_buf: vec![],
    };
    let decoded_value = buf.decode()?;
    display_value(&decoded_value);

    let info_value = dict_get(&decoded_value, "info")?;

    let announce_vec = dict_get_as(&decoded_value, "announce", value_as_bytes)?;

    let length = dict_get_as(&info_value, "length", value_as_int)?;
    let piece_length = dict_get_as(&info_value, "piece length", value_as_int)?;

    let _ = buf.encode(&info_value);

    let _sh1_digest = Sha1::digest(&buf.outer_buf);
    let hx = hex::encode(_sh1_digest);

    println!("Tracker URL: {}", String::from_utf8(announce_vec)?);
    println!("Length: {}", length);
    println!("Info Hash: {} ", hx);
    println!("Piece Length: {}", piece_length);
    println!("Piece Hashes: \n{}", pieces_hash(&info_value)?.join("\n"));
    Ok(())
}

pub fn info_task<T: AsRef<Path>>(file_name: T) -> MyTorrentResult<()> {
    // info_raw(file_name)?;
    let b = MyTorrent::from_file(file_name);
    // let m = MyPeerMsg::request(1, 2, 3);
    println!("Tracker URL: {}", b.announce);
    match &b.info.keys {
        crate::my_impl::MyTorrentInfoKeys::SingleFile { length } => {
            println!("Length: {:?}", length)
        }
        crate::my_impl::MyTorrentInfoKeys::MultiFile { files } => {
            files
                .iter()
                .for_each(|f| println!("Length: {:?}", f.length));
        }
    }
    println!("Info Hash: {} ", b.info.hash());
    println!("Piece Length: {}", b.info.piece_length);
    println!("Piece Hashes: ");
    b.info.pieces.print();
    Ok(())
}
