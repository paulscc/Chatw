use crate::redis_client::RedisClient;

pub async fn test_redis_connection() -> anyhow::Result<()> {
    println!("Testing Redis connection...");
    
    let redis_client = RedisClient::new("redis://localhost:6379").await?;
    
    // Test PING
    let ping_result = redis_client.test_connection().await?;
    println!("Redis PING: {}", ping_result);
    
    // Test SET/GET
    let test_key = "test_redis_key";
    let test_value = "Hello Redis!";
    
    redis_client.set(test_key, &test_value, None).await?;
    println!("SET {} = {}", test_key, test_value);
    
    let retrieved: Option<String> = redis_client.get(test_key).await?;
    println!("GET {}: {:?}", test_key, retrieved);
    
    // Test DELETE
    let deleted = redis_client.delete(test_key).await?;
    println!("DELETE {}: {}", test_key, deleted);
    
    // Test INCR/DECR
    let counter_key = "test_counter";
    let count = redis_client.increment(counter_key).await?;
    println!("INCR {}: {}", counter_key, count);
    
    let count = redis_client.decrement(counter_key).await?;
    println!("DECR {}: {}", counter_key, count);
    
    redis_client.delete(counter_key).await?;
    
    println!("Redis tests completed successfully!");
    Ok(())
}
