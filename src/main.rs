use anyhow::{bail, Context, Result};
use bittorrent_starter_rust::{
    dict_get, dict_get_as, display_value, torrent_task, value_as_bytes, value_as_dict,
};
use serde::{Deserialize, Serialize};



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = torrent_task().await;
    Ok(())
}
#[derive(Debug, Deserialize, Serialize)]
struct MyStruct {
    pub name: i64,
    #[serde(flatten)]
    pub aa: MyStruct_aa,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum MyStruct_aa {
    a1 { xx: String },
    a2 { gg: i32 },
}
#[test]
fn ta() -> Result<()> {
    let m = MyStruct {
        name: 22,
        aa: MyStruct_aa::a1 {
            xx: "??".to_owned(),
        },
    };
    let response = serde_json::to_string(&m);
    let response: MyStruct = serde_json::from_str(
        r#"
        {"name":22,"gg":2}
        "#,
    )
    .context("parse tracker response")?;
    println!("get response {:?}", response);
    Ok(())
}
