use etcd_client;

pub trait Database {
    async fn new() -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
    async fn put(&mut self, key: &str, value: &str);
    async fn get_value(&mut self, key: &str) -> Option<String>;
    async fn clear(&mut self);
}

pub struct Etcd {
    client: etcd_client::Client,
}

impl Database for Etcd {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = etcd_client::Client::connect(["http://127.0.0.1:2379"], None).await?;
        Ok(Etcd { client })
    }

    async fn put(&mut self, k: &str, value: &str) {
        let key = format!("/v1/{}", k);
        self.client.put(key, value, None).await.unwrap();
    }

    async fn get_value(&mut self, k: &str) -> Option<String> {
        let key = format!("/v1/{}", k);

        let resp = self.client.get(key, None).await.unwrap();
        if let Some(kv) = resp.kvs().first() {
            return Some(kv.value_str().unwrap().to_string());
        }
        None
    }
    async fn clear(&mut self) {
        //delete all keys
        let resp = self.client.get("/v1/", None).await.unwrap();
        for kv in resp.kvs() {
            self.client
                .delete(kv.key_str().unwrap(), None)
                .await
                .unwrap();
        }
    }
}
