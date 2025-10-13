pub mod openai;
pub mod claude;

use crate::types::{ChatCompletionObject, ChatRequest};
use data_encoding::HEXLOWER;
use ring::digest;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::{anyhow, bail};
use crate::json::{ToJson, FromJson};
pub trait RequestCache {
    //TODO: Use a better Result type!
    fn get_response_if_cached(
        &self,
        request: &ChatRequest,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<ChatCompletionObject>>> + Send>>;
    fn cache_response<'a>(
        &'a self,
        request: &'a ChatRequest,
        response: &'a ChatCompletionObject,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;
}

#[derive(Debug, PartialEq)]
pub enum TrivialFSPathType {
    NoSuchPath,
    File,
    Directory,
}

use std::future::Future;
use std::pin::Pin;

pub trait TrivialFS {
    fn read_to_string(&self, p: &Path) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>;
    fn write<'a>(&'a self, p: &'a Path, value: &'a str) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;
    fn path_type(&self, p: &Path) -> Pin<Box<dyn Future<Output = anyhow::Result<TrivialFSPathType>> + Send>>;
}

pub struct DefaultFS {}

impl TrivialFS for DefaultFS {
    fn read_to_string(&self, p: &Path) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>> {
        let p = p.to_path_buf();
        Box::pin(async move {
            Ok(tokio::fs::read_to_string(p).await?)
        })
    }
    fn write<'a>(&'a self, p: &'a Path, value: &'a str) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            tokio::fs::write(p, value).await?;
            Ok(())
        })
    }
    fn path_type(&self, p: &Path) -> Pin<Box<dyn Future<Output = anyhow::Result<TrivialFSPathType>> + Send>> {
        let p = p.to_path_buf();
        Box::pin(async move {
            use std::io::ErrorKind;

            let r = tokio::fs::metadata(p).await;
            match r {
                Ok(metadata) => {
                    if metadata.is_file() {
                        Ok(TrivialFSPathType::File)
                    } else if metadata.is_dir() {
                        Ok(TrivialFSPathType::Directory)
                    } else {
                        Err(anyhow!(
                            "path_type failed: path is not a file or directory"
                        ))
                    }
                }
                Err(e) if e.kind() == ErrorKind::NotFound => Ok(TrivialFSPathType::NoSuchPath),
                Err(e) => Err(anyhow!(
                    "path_type failed when stating path: {}",
                    e
                )),
            }
        })
    }
}

pub struct DefaultRequestCache {
    fs: Arc<DefaultFS>,
    root: PathBuf,
}

impl DefaultRequestCache {
    pub async fn new(
        fs: Arc<DefaultFS>,
        root: PathBuf,
    ) -> anyhow::Result<DefaultRequestCache> {
        let r = fs.path_type(&root).await?;
        if r != TrivialFSPathType::Directory {
            bail!(
                "DefaultRrequestCache::new failed - '{}' is not a directory",
                root.display()
            );
        }
        Ok(DefaultRequestCache { fs, root })
    }

    fn key(&self, value: &ChatRequest) -> String {
        let request_json = value.to_json();
        let request_str = request_json.to_string();
        let digest = digest::digest(&digest::SHA256, request_str.as_bytes());
        // The key length is way too big for what we want.
        let full_key = HEXLOWER.encode(digest.as_ref());
        let key = &full_key[0..32];
        key.to_string()
    }

    fn key_to_path(&self, key: &str) -> PathBuf {
        self.root.join(format!("{}.json", key))
    }
}

impl RequestCache for DefaultRequestCache {
    fn get_response_if_cached(
        &self,
        request: &ChatRequest,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<ChatCompletionObject>>> + Send>> {
        let key = self.key(request);
        let cache_file_path = self.key_to_path(&key);
        let fs = self.fs.clone();
        let request = request.clone();
        Box::pin(async move {
            if let Ok(content) = fs.read_to_string(&cache_file_path).await {
                let value: serde_json::Value = serde_json::from_str(&content)?;
                let cached_request = ChatRequest::from_json(&value["request"])
                    .map_err(|_e| anyhow!("unable to decode request"))?;
                let cached_response = ChatCompletionObject::from_json(&value["response"])
                    .map_err(|_e| anyhow!("unabled to decode resposne"))?;
                if cached_request != request {
                    anyhow::bail!("Cached request does not match!");
                }
                Ok(Some(cached_response))
            } else {
                Ok(None)
            }
        })
    }

    fn cache_response<'a>(
        &'a self,
        request: &'a ChatRequest,
        response: &'a ChatCompletionObject,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        let key = self.key(request);
        let cache_file_path = self.key_to_path(&key);
        let cache_entry = json!({
            "request": request.to_json(),
            "response": response.to_json(),
        });
        let fs = self.fs.clone();
        Box::pin(async move {
            fs.write(
                &cache_file_path,
                &serde_json::to_string_pretty(&cache_entry).unwrap(),
            )
            .await?;
            Ok(())
        })
    }
}
