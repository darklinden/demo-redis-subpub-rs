use anyhow::Result;
use arc_swap::ArcSwap;
use futures_util::StreamExt;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

#[derive(Debug)]
pub struct Consumer {
    data: ArcSwap<HashMap<String, String>>,
}

static CONSUMER_INSTANCE: OnceLock<Consumer> = OnceLock::new();

impl Consumer {
    pub async fn reload_data(&self, msg: &str) -> Result<()> {
        println!("Doing something start with message: {}", msg);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        println!("Doing something end with message: {}", msg);

        let data = serde_json::from_str::<HashMap<String, String>>(msg).unwrap();
        println!("Data: {:?}", data);
        self.data.swap(Arc::new(data));

        Ok(())
    }

    pub fn instance() -> &'static Consumer {
        CONSUMER_INSTANCE.get_or_init(|| Consumer {
            data: ArcSwap::new(Arc::new(HashMap::new())),
        })
    }

    pub fn get(key: &str) -> Option<String> {
        Self::instance().data.load().get(key).cloned()
    }

    pub fn subscribe(rd: Arc<redis::Client>, channel_name: &str) {
        let channel_name = channel_name.to_owned();
        tokio::spawn(async move {
            let result = Consumer::instance()
                .async_subscribe(rd, &channel_name)
                .await;
            if let Err(err) = result {
                log::error!("Error subscribing to channel: {}", err);
            }
        });
    }

    async fn async_subscribe(&self, rd: Arc<redis::Client>, channel_name: &str) -> Result<()> {
        let mut pubsub = rd.get_async_pubsub().await?;
        pubsub.subscribe(channel_name).await?;

        // Inside the spawned task, use a loop to continuously process messages
        while let Some(msg) = pubsub.on_message().next().await {
            println!("Received message: {:?}", msg);
            let json_str = msg.get_payload::<String>().unwrap();
            self.reload_data(&json_str).await?;
        }

        Ok(())
    }
}
