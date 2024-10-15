use anyhow::Result;

use crate::my_impl::MyMagnet;

pub fn magnet_parse_task(link: &str) ->Result<()>{
    let m = MyMagnet::from_link(link)?;
    println!("Tracker URL: {}",m.tr);
    println!("Info Hash: {}",m.urn_btih);
    Ok(())
}
