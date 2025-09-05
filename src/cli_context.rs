use crate::client::LinearClient;
use crate::config::{get_api_key, save_config, load_config};
use crate::error::{LinearError, LinearResult};
use std::sync::Arc;

/// Central context for CLI operations, managing configuration and client instances
pub struct CliContext {
    api_key: Option<String>,
    client: Option<Arc<LinearClient>>,
}

impl CliContext {
    /// Create a new CLI context
    pub fn new() -> Self {
        Self {
            api_key: None,
            client: None,
        }
    }
    
    /// Load context from saved configuration
    pub fn load() -> LinearResult<Self> {
        let api_key = get_api_key().ok();
        let client = api_key.as_ref().map(|key| Arc::new(LinearClient::new(key.clone())));
        
        Ok(Self { api_key, client })
    }
    
    /// Get or create a verified client (requires API key)
    pub fn verified_client(&mut self) -> LinearResult<Arc<LinearClient>> {
        if let Some(client) = &self.client {
            return Ok(client.clone());
        }
        
        let api_key = self.api_key()?.clone();
        let client = Arc::new(LinearClient::new(api_key));
        self.client = Some(client.clone());
        Ok(client)
    }
    
    /// Get or create an unverified client (creates one if API key is available)
    pub fn unverified_client(&mut self) -> Option<Arc<LinearClient>> {
        if let Some(client) = &self.client {
            return Some(client.clone());
        }
        
        if let Ok(api_key) = self.api_key() {
            let client = Arc::new(LinearClient::new(api_key.clone()));
            self.client = Some(client.clone());
            return self.client.clone();
        }
        
        None
    }
    
    /// Get the API key, loading from config if necessary
    pub fn api_key(&mut self) -> LinearResult<&String> {
        if self.api_key.is_none() {
            self.api_key = Some(get_api_key().map_err(|_| LinearError::ApiKeyNotFound)?);
        }
        
        self.api_key.as_ref().ok_or(LinearError::ApiKeyNotFound)
    }
    
    /// Set and save a new API key
    pub fn set_api_key(&mut self, api_key: String) -> LinearResult<()> {
        let mut config = load_config();
        config.api_key = Some(api_key.clone());
        save_config(&config).map_err(|e| LinearError::ConfigError(e.to_string()))?;
        self.api_key = Some(api_key.clone());
        self.client = Some(Arc::new(LinearClient::new(api_key)));
        Ok(())
    }
    
    /// Check if context has a valid API key
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some() || get_api_key().is_ok()
    }
}

/// Builder pattern for creating CLI contexts with specific configurations
pub struct CliContextBuilder {
    api_key: Option<String>,
}

impl CliContextBuilder {
    pub fn new() -> Self {
        Self { api_key: None }
    }
    
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
    
    pub fn build(self) -> LinearResult<CliContext> {
        let context = if let Some(api_key) = self.api_key {
            let client = Some(Arc::new(LinearClient::new(api_key.clone())));
            CliContext {
                api_key: Some(api_key),
                client,
            }
        } else {
            CliContext::load()?
        };
        
        Ok(context)
    }
}

impl Default for CliContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}