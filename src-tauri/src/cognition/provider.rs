use crate::storage::SharedStorage;
use crate::cognition::routing::ModelCapability;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tracing::{info, warn, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredProvider {
    pub id: String,
    pub name: String,
    pub provider_type: String, // "ollama" or "openai"
    pub api_url: String,
    pub api_key: Option<String>,
    pub is_enabled: bool,
    pub capabilities: Vec<ModelCapability>,
    pub routing_priority: i32,
    pub model_name: String,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn generate(&self, prompt: &str, system_prompt: Option<&str>, max_tokens: Option<usize>) -> Result<String>;
    fn name(&self) -> &str;
    fn api_url(&self) -> &str;
    fn get_capabilities(&self) -> Vec<ModelCapability>;
    fn model_name(&self) -> &str;
}

pub struct OllamaProvider {
    pub name: String,
    pub api_url: String,
    pub model_name: String,
    pub capabilities: Vec<ModelCapability>,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(name: String, api_url: String, model_name: String, capabilities: Vec<ModelCapability>) -> Self {
        Self {
            name,
            api_url,
            model_name,
            capabilities,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ModelProvider for OllamaProvider {
    #[instrument(skip(self, prompt, system_prompt))]
    async fn generate(&self, prompt: &str, system_prompt: Option<&str>, max_tokens: Option<usize>) -> Result<String> {
        let endpoint = format!("{}/api/generate", self.api_url.trim_end_matches('/'));
        info!(endpoint = %endpoint, model = %self.model_name, "Calling local Ollama provider");

        let mut body = serde_json::json!({
            "model": self.model_name,
            "prompt": prompt,
            "stream": false,
        });

        if let Some(sys) = system_prompt {
            body["system"] = serde_json::json!(sys);
        }

        if let Some(tokens) = max_tokens {
            body["options"] = serde_json::json!({
                "num_predict": tokens
            });
        }

        let res = self.client.post(&endpoint)
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
            let status_code = res.status();
            let err_text = res.text().await?;
            warn!(status = ?status_code, err = %err_text, "Ollama request failed");
            return Err(anyhow!("Ollama error ({}): {}", status_code, err_text));
        }

        let payload: serde_json::Value = res.json().await?;
        let text = payload["response"].as_str()
            .ok_or_else(|| anyhow!("Invalid Ollama payload schema"))?
            .to_string();

        Ok(text)
    }

    fn name(&self) -> &str { &self.name }
    fn api_url(&self) -> &str { &self.api_url }
    fn get_capabilities(&self) -> Vec<ModelCapability> { self.capabilities.clone() }
    fn model_name(&self) -> &str { &self.model_name }
}

pub struct OpenAiCompatibleProvider {
    pub name: String,
    pub api_url: String,
    pub api_key: Option<String>,
    pub model_name: String,
    pub capabilities: Vec<ModelCapability>,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(name: String, api_url: String, api_key: Option<String>, model_name: String, capabilities: Vec<ModelCapability>) -> Self {
        Self {
            name,
            api_url,
            api_key,
            model_name,
            capabilities,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ModelProvider for OpenAiCompatibleProvider {
    #[instrument(skip(self, prompt, system_prompt))]
    async fn generate(&self, prompt: &str, system_prompt: Option<&str>, max_tokens: Option<usize>) -> Result<String> {
        let endpoint = format!("{}/chat/completions", self.api_url.trim_end_matches('/'));
        info!(endpoint = %endpoint, model = %self.model_name, "Calling OpenAI Compatible provider");

        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(serde_json::json!({
                "role": "system",
                "content": sys
            }));
        }
        messages.push(serde_json::json!({
            "role": "user",
            "content": prompt
        }));

        let mut body = serde_json::json!({
            "model": self.model_name,
            "messages": messages,
            "stream": false
        });

        if let Some(tokens) = max_tokens {
            body["max_tokens"] = serde_json::json!(tokens);
        }

        let mut req = self.client.post(&endpoint).json(&body);

        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let res = req.send().await?;

        if !res.status().is_success() {
            let status_code = res.status();
            let err_text = res.text().await?;
            warn!(status = ?status_code, err = %err_text, "OpenAI Compatible request failed");
            return Err(anyhow!("OpenAI error ({}): {}", status_code, err_text));
        }

        let payload: serde_json::Value = res.json().await?;
        let text = payload["choices"][0]["message"]["content"].as_str()
            .ok_or_else(|| anyhow!("Invalid OpenAI Compatible payload schema"))?
            .to_string();

        Ok(text)
    }

    fn name(&self) -> &str { &self.name }
    fn api_url(&self) -> &str { &self.api_url }
    fn get_capabilities(&self) -> Vec<ModelCapability> { self.capabilities.clone() }
    fn model_name(&self) -> &str { &self.model_name }
}

pub struct ProviderRegistry {
    storage: SharedStorage,
}

impl ProviderRegistry {
    pub fn new(storage: SharedStorage) -> Self {
        Self { storage }
    }

    #[instrument(skip(self))]
    pub fn add_provider(&self, provider: RegisteredProvider) -> Result<()> {
        info!(id = %provider.id, name = %provider.name, "Adding provider to registry");
        let conn = self.storage.conn.lock().unwrap();

        let capabilities_json = serde_json::to_string(&provider.capabilities)?;

        // Simple obfuscation of API key in credentials table
        let obscured_key = provider.api_key.map(|k| {
            k.chars().map(|c| ((c as u32) + 1) as u8 as char).collect::<String>()
        });

        conn.execute(
            "INSERT INTO provider_configs (id, name, api_url, api_key, is_enabled, capabilities, routing_priority, model_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                api_url = excluded.api_url,
                api_key = excluded.api_key,
                is_enabled = excluded.is_enabled,
                capabilities = excluded.capabilities,
                routing_priority = excluded.routing_priority,
                model_name = excluded.model_name",
            (
                &provider.id,
                &provider.name,
                &provider.api_url,
                obscured_key,
                if provider.is_enabled { 1 } else { 0 },
                capabilities_json,
                provider.routing_priority,
                &provider.model_name,
            ),
        )?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn remove_provider(&self, id: &str) -> Result<()> {
        info!(id = %id, "Removing provider from registry");
        let conn = self.storage.conn.lock().unwrap();
        conn.execute("DELETE FROM provider_configs WHERE id = ?1", [id])?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_providers(&self) -> Result<Vec<RegisteredProvider>> {
        let conn = self.storage.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, api_url, api_key, is_enabled, capabilities, routing_priority, model_name FROM provider_configs"
        )?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let api_url: String = row.get(2)?;
            let api_key_obscured: Option<String> = row.get(3)?;
            let is_enabled: i32 = row.get(4)?;
            let capabilities_raw: String = row.get(5)?;
            let routing_priority: i32 = row.get(6)?;
            let model_name: String = row.get(7)?;

            let capabilities: Vec<ModelCapability> = serde_json::from_str(&capabilities_raw)
                .unwrap_or_else(|_| vec![]);

            let api_key = api_key_obscured.map(|k| {
                k.chars().map(|c| ((c as u32) - 1) as u8 as char).collect::<String>()
            });

            // Derive provider type from configuration
            let provider_type = if api_url.contains("openai") || api_key.is_some() {
                "openai".to_string()
            } else {
                "ollama".to_string()
            };

            Ok(RegisteredProvider {
                id,
                name,
                provider_type,
                api_url,
                api_key,
                is_enabled: is_enabled != 0,
                capabilities,
                routing_priority,
                model_name,
            })
        })?;

        let mut providers = Vec::new();
        for row in rows {
            providers.push(row?);
        }

        Ok(providers)
    }

    pub fn get_provider_adapter(&self, id: &str) -> Result<Arc<dyn ModelProvider>> {
        let providers = self.get_providers()?;
        let p = providers.into_iter()
            .find(|x| x.id == id)
            .ok_or_else(|| anyhow!("Provider {} not found", id))?;

        if !p.is_enabled {
            return Err(anyhow!("Provider {} is disabled", id));
        }

        if p.provider_type == "ollama" {
            Ok(Arc::new(OllamaProvider::new(p.name, p.api_url, p.model_name, p.capabilities)))
        } else {
            Ok(Arc::new(OpenAiCompatibleProvider::new(p.name, p.api_url, p.api_key, p.model_name, p.capabilities)))
        }
    }
}
