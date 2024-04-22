use anyhow::Result;
use redis::Commands;
use std::sync::Arc;

mod consumer;
use crate::consumer::Consumer;

fn publish(rd: Arc<redis::Client>, channel_name: &str, payload: &str) -> Result<()> {
    let mut conn = rd.get_connection()?;
    conn.publish(channel_name, payload)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let redis = Arc::new(redis::Client::open("redis://127.0.0.1:6379").unwrap());
    let channel_name = "channel";

    Consumer::subscribe(redis.clone(), channel_name);

    tokio::spawn(async move {
        let mut msg_id = 0_i64;

        loop {
            msg_id += 1;
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            let result = publish(
                redis.clone(),
                channel_name,
                &format!("{{\"name\":\"Hello\", \"id\":\"{}\"}}", msg_id),
            );

            println!("get message: {:?}", Consumer::get("id"));

            if let Err(err) = result {
                log::error!("Error publishing message: {}", err);
            }
        }
    })
    .await
    .unwrap();
}
