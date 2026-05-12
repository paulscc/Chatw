use redis::{Client, RedisError, RedisResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        
        Ok(Self { client })
    }

    pub async fn test_connection(&self) -> RedisResult<String> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        redis::cmd("PING").query_async(&mut conn).await
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> RedisResult<()> {
        let serialized = serde_json::to_string(value).map_err(|e| {
            RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string()))
        })?;

        let mut conn = self.client.get_multiplexed_async_connection().await?;
        
        if let Some(ttl) = ttl_seconds {
            redis::cmd("SETEX")
                .arg(key)
                .arg(ttl)
                .arg(serialized)
                .query_async::<_, ()>(&mut conn)
                .await?;
        } else {
            redis::cmd("SET")
                .arg(key)
                .arg(serialized)
                .query_async::<_, ()>(&mut conn)
                .await?;
        }
        
        Ok(())
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> RedisResult<Option<T>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value: Option<String> = redis::cmd("GET").arg(key).query_async(&mut conn).await?;
        
        match value {
            Some(json_str) => {
                let deserialized: T = serde_json::from_str(&json_str).map_err(|e| {
                    RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed", e.to_string()))
                })?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    pub async fn delete(&self, key: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let count: i32 = redis::cmd("DEL").arg(key).query_async(&mut conn).await?;
        Ok(count > 0)
    }

    pub async fn exists(&self, key: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let exists: i32 = redis::cmd("EXISTS").arg(key).query_async(&mut conn).await?;
        Ok(exists > 0)
    }

    pub async fn expire(&self, key: &str, seconds: u64) -> RedisResult<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let result: i32 = redis::cmd("EXPIRE").arg(key).arg(seconds).query_async(&mut conn).await?;
        Ok(result > 0)
    }

    pub async fn increment(&self, key: &str) -> RedisResult<i64> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        redis::cmd("INCR").arg(key).query_async(&mut conn).await
    }

    pub async fn decrement(&self, key: &str) -> RedisResult<i64> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        redis::cmd("DECR").arg(key).query_async(&mut conn).await
    }

    pub async fn add_to_set<T: Serialize>(&self, key: &str, value: &T) -> RedisResult<bool> {
        let serialized = serde_json::to_string(value).map_err(|e| {
            RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string()))
        })?;

        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let result: i32 = redis::cmd("SADD").arg(key).arg(serialized).query_async(&mut conn).await?;
        Ok(result > 0)
    }

    pub async fn remove_from_set<T: Serialize>(&self, key: &str, value: &T) -> RedisResult<bool> {
        let serialized = serde_json::to_string(value).map_err(|e| {
            RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string()))
        })?;

        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let result: i32 = redis::cmd("SREM").arg(key).arg(serialized).query_async(&mut conn).await?;
        Ok(result > 0)
    }

    pub async fn is_in_set<T: Serialize>(&self, key: &str, value: &T) -> RedisResult<bool> {
        let serialized = serde_json::to_string(value).map_err(|e| {
            RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string()))
        })?;

        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let result: i32 = redis::cmd("SISMEMBER").arg(key).arg(serialized).query_async(&mut conn).await?;
        Ok(result > 0)
    }

    pub async fn publish(&self, channel: &str, message: &str) -> RedisResult<i64> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let result: i64 = redis::cmd("PUBLISH").arg(channel).arg(message).query_async(&mut conn).await?;
        Ok(result)
    }

    pub async fn subscribe(&self, channel: &str) -> RedisResult<redis::aio::PubSub> {
        let conn = self.client.get_async_connection().await?;
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe(channel).await?;
        Ok(pubsub)
    }
}

