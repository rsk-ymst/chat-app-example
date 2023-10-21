#![allow(unused)]

use anyhow::Result;
use serde_json::json;

use uuid::Uuid;


#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("ws://0.0.0.0:8080")?;

    hc.do_get("/ws").await?.print().await?;

    Ok(())
}

#[tokio::test]
async fn create_uuid_from_byte() -> Result<()> {

    let entry_room_id = Uuid::from_slice(b"entry").unwrap();

    Ok(())
}

#[tokio::test]
async fn create_uuid_from_same_byte() -> Result<()> {

    let hc = httpc_test::new_client("ws://0.0.0.0:8080")?;
    let entry_room_id = Uuid::from_slice(b"entry").unwrap();


    hc.do_get("/ws").await?.print().await?;

    Ok(())
}
