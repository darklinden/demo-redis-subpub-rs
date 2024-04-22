use std::sync::Arc;

use anyhow::Result;
use futures_util::StreamExt;

#[derive(Debug, Clone)]
pub struct Consumer {}

impl Default for Consumer {
    fn default() -> Self {
        Self {}
    }
}

impl Consumer {
    pub async fn do_something(&self, msg: &str) {
        println!("Doing something start with message: {}", msg);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        println!("Doing something end with message: {}", msg);
    }
}

pub fn subscribe(consumer: Arc<Consumer>, rd: Arc<redis::Client>, channel_name: &str) {
    let channel_name = channel_name.to_owned();
    tokio::spawn(async move {
        let result = async_subscribe(consumer, rd, &channel_name).await;
        if let Err(err) = result {
            log::error!("Error subscribing to channel: {}", err);
        }
    });
}

async fn async_subscribe(
    consumer: Arc<Consumer>,
    rd: Arc<redis::Client>,
    channel_name: &str,
) -> Result<()> {
    let mut pubsub = rd.get_async_pubsub().await?;
    pubsub.subscribe(channel_name).await?;

    // Inside the spawned task, use a loop to continuously process messages
    while let Some(msg) = pubsub.on_message().next().await {
        println!("Received message: {:?}", msg);
        consumer
            .do_something(&msg.get_payload::<String>().unwrap())
            .await;
    }

    Ok(())
}
