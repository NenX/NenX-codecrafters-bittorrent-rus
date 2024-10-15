use anyhow::Result;

use crate::my_impl::MyMagnet;

pub fn magnet_parse_task(link: &str) -> Result<()> {
    let m = MyMagnet::from_link(link)?;
    m.print();
    Ok(())
}
