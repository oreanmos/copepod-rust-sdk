use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

/// A token pair (access + refresh).
#[derive(Debug, Clone)]
pub struct TokenPair {
    pub token: String,
    pub refresh_token: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Thread-safe token store used by `CopepodClient` for automatic auth management.
#[derive(Debug, Default)]
pub struct TokenStore {
    inner: RwLock<Option<TokenPair>>,
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(None),
        }
    }

    /// Create a new token store pre-loaded with a token pair.
    pub fn with_token(pair: TokenPair) -> Self {
        Self {
            inner: RwLock::new(Some(pair)),
        }
    }

    /// Store a new token pair.
    pub async fn set(&self, pair: TokenPair) {
        let mut guard = self.inner.write().await;
        *guard = Some(pair);
    }

    /// Retrieve the current token pair, if any.
    pub async fn get(&self) -> Option<TokenPair> {
        self.inner.read().await.clone()
    }

    /// Clear the stored tokens.
    pub async fn clear(&self) {
        let mut guard = self.inner.write().await;
        *guard = None;
    }

    /// Check if the current token has expired.
    pub async fn is_expired(&self) -> bool {
        let guard = self.inner.read().await;
        match &*guard {
            None => true,
            Some(pair) => match pair.expires_at {
                None => false,
                Some(exp) => Utc::now() >= exp,
            },
        }
    }

    /// Check if the token should be refreshed (expires within 60 seconds).
    pub async fn needs_refresh(&self) -> bool {
        let guard = self.inner.read().await;
        match &*guard {
            None => false,
            Some(pair) => match pair.expires_at {
                None => false,
                Some(exp) => {
                    let threshold = exp - chrono::Duration::seconds(60);
                    Utc::now() >= threshold
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_token_store_set_get_clear() {
        let store = TokenStore::new();
        assert!(store.get().await.is_none());

        store
            .set(TokenPair {
                token: "tok".into(),
                refresh_token: "ref".into(),
                expires_at: None,
            })
            .await;
        let pair = store.get().await.unwrap();
        assert_eq!(pair.token, "tok");

        store.clear().await;
        assert!(store.get().await.is_none());
    }

    #[tokio::test]
    async fn test_is_expired() {
        let store = TokenStore::new();
        // No token → expired
        assert!(store.is_expired().await);

        // Token with no expiry → not expired
        store
            .set(TokenPair {
                token: "t".into(),
                refresh_token: "r".into(),
                expires_at: None,
            })
            .await;
        assert!(!store.is_expired().await);

        // Token expired in the past
        store
            .set(TokenPair {
                token: "t".into(),
                refresh_token: "r".into(),
                expires_at: Some(Utc::now() - Duration::seconds(10)),
            })
            .await;
        assert!(store.is_expired().await);

        // Token expires in the future
        store
            .set(TokenPair {
                token: "t".into(),
                refresh_token: "r".into(),
                expires_at: Some(Utc::now() + Duration::hours(1)),
            })
            .await;
        assert!(!store.is_expired().await);
    }

    #[tokio::test]
    async fn test_needs_refresh() {
        let store = TokenStore::new();
        // No token → no refresh needed
        assert!(!store.needs_refresh().await);

        // Expires in 30s → needs refresh
        store
            .set(TokenPair {
                token: "t".into(),
                refresh_token: "r".into(),
                expires_at: Some(Utc::now() + Duration::seconds(30)),
            })
            .await;
        assert!(store.needs_refresh().await);

        // Expires in 2 hours → no refresh needed
        store
            .set(TokenPair {
                token: "t".into(),
                refresh_token: "r".into(),
                expires_at: Some(Utc::now() + Duration::hours(2)),
            })
            .await;
        assert!(!store.needs_refresh().await);
    }
}
