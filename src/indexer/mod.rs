use crate::database;
use base64::prelude::*;
use std::fs;
use std::io;
use walkdir::WalkDir;

use database::Database;
use log::debug;
use std::os::unix::fs::MetadataExt;
pub struct Indexer<T: database::Database> {
    database: T,
}

pub struct Differences {
    pub changed: Vec<String>,
}

impl Indexer<database::Etcd> {
    pub fn new(database: database::Etcd) -> Indexer<database::Etcd> {
        Indexer { database }
    }

    fn system_time_to_unix_timestamp(&self, time: std::time::SystemTime) -> u64 {
        time.duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    async fn build_key(&self, path: &str) -> String {
        let meta = fs::metadata(path).unwrap();
        let ino = meta.ino();
        let created = self.system_time_to_unix_timestamp(meta.created().unwrap());

        let key = format!("{}:{}:{}", ino, created.to_string(), path);
        BASE64_STANDARD.encode(key.as_bytes())
    }
    pub async fn clear(&mut self) {
        self.database.clear().await;
        println!("Cleared the database");
    }
    fn deconstruct_key(&self, key: &str) -> (u64, u64, String) {
        let decoded = BASE64_STANDARD.decode(key.as_bytes()).unwrap();
        let decoded = String::from_utf8(decoded).unwrap();
        let parts: Vec<&str> = decoded.split(":").collect();
        let ino = parts[0].parse::<u64>().unwrap();
        let created = parts[1].parse::<u64>().unwrap();
        let path = parts[2].to_string();
        (ino, created, path)
    }

    pub async fn index(&mut self, path: &str, check: bool) -> Option<Differences> {
        let mut differences = Differences {
            changed: Vec::new(),
        };
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let hash = format!("{:x}", md5::compute(fs::read(entry.path()).unwrap()));
            let key = self.build_key(entry.path().to_str().unwrap()).await;
            // check if the file is already indexed
            let r = self.database.get_value(&key).await;
            match r {
                Some(v) => {
                    // if the file is indexed, check to see if the value matches
                    if hash == v {
                        debug!(
                            "Skipping {} because it's already indexed",
                            entry.path().to_str().unwrap()
                        );
                    } else if check {
                        differences
                            .changed
                            .push(entry.path().to_str().unwrap().to_string());
                        continue;
                    }
                    let (ino, created, path) = self.deconstruct_key(&key);
                    let meta = fs::metadata(path).unwrap();
                    let new_ino = meta.ino();
                    let new_created = self.system_time_to_unix_timestamp(meta.created().unwrap());
                    if ino != new_ino || created != new_created {
                        let hash = format!("{:x}", md5::compute(fs::read(entry.path()).unwrap()));
                        debug!(
                            "Re-indexing {} with hash {}",
                            entry.path().to_str().unwrap(),
                            hash
                        );
                        self.database.put(&key, &hash).await;
                    }
                }
                None => {
                    debug!(
                        "Indexing {} with hash {}",
                        entry.path().to_str().unwrap(),
                        hash
                    );
                    self.database.put(&key, &hash).await;
                }
            }
        }
        if differences.changed.len() > 0 {
            Some(differences)
        } else {
            None
        }
    }
}
