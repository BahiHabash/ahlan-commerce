pub mod keys;

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

/// A shared Redis cache client.
///
/// All methods are fail-safe: any Redis error is logged and the caller receives
/// a `None` / `false` so the application can fall back to the database without
/// crashing.
#[derive(Clone)]
pub struct Cache {
    client: redis::Client,
}

impl Cache {
    /// Create a new `Cache` from a Redis connection URL
    /// (e.g. `"redis://127.0.0.1:6379"`).
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    /// Attempt to get a JSON-deserialised value from Redis.
    ///
    /// Returns `Some(value)` on a cache hit, `None` on a miss, parse failure,
    /// or any Redis error.
    ///
    /// # Logs
    /// - `cache_hit` — key was found and parsed successfully.
    /// - `cache_miss` — key was not found.
    /// - `redis_unavailable_fallback` — Redis returned an error.
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, cache_key: &str) -> Option<T> {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(err) => {
                tracing::warn!(
                    cache_key = %cache_key,
                    error = %err,
                    "redis_unavailable_fallback: failed to acquire connection on get"
                );
                return None;
            }
        };

        let raw: Option<String> = match conn.get(cache_key).await {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(
                    cache_key = %cache_key,
                    error = %err,
                    "redis_unavailable_fallback: GET command failed"
                );
                return None;
            }
        };

        match raw {
            None => {
                tracing::info!(cache_key = %cache_key, "cache_miss");
                None
            }
            Some(json) => match serde_json::from_str::<T>(&json) {
                Ok(value) => {
                    tracing::info!(cache_key = %cache_key, "cache_hit");
                    Some(value)
                }
                Err(err) => {
                    tracing::warn!(
                        cache_key = %cache_key,
                        error = %err,
                        "cache_miss: JSON parse failed, treating as miss"
                    );
                    None
                }
            },
        }
    }

    /// Store a JSON-serialised value in Redis with a TTL (in seconds).
    ///
    /// Returns `true` if the value was stored successfully, `false` otherwise.
    ///
    /// # Logs
    /// - `cache_set_failure` — Redis SET command failed.
    /// - `redis_unavailable_fallback` — Redis connection could not be acquired.
    pub async fn set<T: Serialize>(&self, cache_key: &str, value: &T, ttl_secs: u64) -> bool {
        let json = match serde_json::to_string(value) {
            Ok(j) => j,
            Err(err) => {
                tracing::warn!(
                    cache_key = %cache_key,
                    error = %err,
                    "cache_set_failure: JSON serialisation failed"
                );
                return false;
            }
        };

        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(err) => {
                tracing::warn!(
                    cache_key = %cache_key,
                    error = %err,
                    "redis_unavailable_fallback: failed to acquire connection on set"
                );
                return false;
            }
        };

        let result: Result<(), redis::RedisError> = conn
            .set_ex(cache_key, &json, ttl_secs)
            .await;

        match result {
            Ok(_) => true,
            Err(err) => {
                tracing::warn!(
                    cache_key = %cache_key,
                    error = %err,
                    "cache_set_failure: SET EX command failed"
                );
                false
            }
        }
    }

    /// Delete a key from Redis.
    ///
    /// Returns `true` if the key was deleted (or did not exist), `false` on
    /// any Redis error.
    ///
    /// # Logs
    /// - `cache_delete_failure` — DEL command failed.
    /// - `redis_unavailable_fallback` — Redis connection could not be acquired.
    pub async fn delete(&self, cache_key: &str) -> bool {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(err) => {
                tracing::warn!(
                    cache_key = %cache_key,
                    error = %err,
                    "redis_unavailable_fallback: failed to acquire connection on delete"
                );
                return false;
            }
        };

        let result: Result<(), redis::RedisError> = conn.del(cache_key).await;

        match result {
            Ok(_) => true,
            Err(err) => {
                tracing::warn!(
                    cache_key = %cache_key,
                    error = %err,
                    "cache_delete_failure: DEL command failed"
                );
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keys::storefront_product_page;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestPayload {
        html: String,
        product_id: String,
    }

    fn test_cache() -> Cache {
        Cache::new("redis://127.0.0.1:6379").expect("Cache::new should succeed with valid URL")
    }

    fn unreachable_cache() -> Cache {
        // Port 16399 is very unlikely to have Redis running.
        Cache::new("redis://127.0.0.1:16399").expect("Cache::new should succeed (URL is valid)")
    }

    // -----------------------------------------------------------------------
    // Normal behaviour — requires a running Redis on localhost:6379
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_miss_returns_none() {
        let cache = test_cache();
        let key = storefront_product_page("no-such-handle");
        // Ensure key does not exist
        cache.delete(&key).await;

        let result: Option<TestPayload> = cache.get(&key).await;
        assert!(result.is_none(), "expected None on cache miss");
    }

    #[tokio::test]
    async fn test_set_and_get_roundtrip() {
        let cache = test_cache();
        let key = storefront_product_page("cache-test-product");
        let payload = TestPayload {
            html: "<html>hello</html>".to_string(),
            product_id: "prod-123".to_string(),
        };

        // Write then read
        let stored = cache.set(&key, &payload, 60).await;
        assert!(stored, "expected set to succeed");

        let retrieved: Option<TestPayload> = cache.get(&key).await;
        assert_eq!(retrieved, Some(payload));

        // Clean up
        cache.delete(&key).await;
    }

    #[tokio::test]
    async fn test_delete_removes_key() {
        let cache = test_cache();
        let key = storefront_product_page("delete-test-product");
        let payload = TestPayload {
            html: "<html>delete me</html>".to_string(),
            product_id: "prod-456".to_string(),
        };

        cache.set(&key, &payload, 60).await;

        let deleted = cache.delete(&key).await;
        assert!(deleted, "expected delete to succeed");

        let after: Option<TestPayload> = cache.get(&key).await;
        assert!(after.is_none(), "expected None after deletion");
    }

    // -----------------------------------------------------------------------
    // Fallback behaviour — Redis unavailable, must not panic
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_returns_none_when_redis_unavailable() {
        let cache = unreachable_cache();
        let key = storefront_product_page("any-handle");

        let result: Option<TestPayload> = cache.get(&key).await;
        assert!(
            result.is_none(),
            "expected None when Redis is unreachable, not a panic"
        );
    }

    #[tokio::test]
    async fn test_set_returns_false_when_redis_unavailable() {
        let cache = unreachable_cache();
        let key = storefront_product_page("any-handle");
        let payload = TestPayload {
            html: "<html/>".to_string(),
            product_id: "x".to_string(),
        };

        let stored = cache.set(&key, &payload, 60).await;
        assert!(
            !stored,
            "expected false when Redis is unreachable, not a panic"
        );
    }

    #[tokio::test]
    async fn test_delete_returns_false_when_redis_unavailable() {
        let cache = unreachable_cache();
        let key = storefront_product_page("any-handle");

        let deleted = cache.delete(&key).await;
        assert!(
            !deleted,
            "expected false when Redis is unreachable, not a panic"
        );
    }

    // -----------------------------------------------------------------------
    // Key format
    // -----------------------------------------------------------------------

    #[test]
    fn test_storefront_product_page_key_format() {
        let key = storefront_product_page("awesome-t-shirt");
        assert_eq!(key, "storefront:product-page:awesome-t-shirt");
    }
}
