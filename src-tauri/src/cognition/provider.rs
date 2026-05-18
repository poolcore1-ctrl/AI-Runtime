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
    pub provider_family: String, // "openai" | "anthropic" | "gemini" | "ollama" | "custom"
    pub price_input_million: f64,
    pub price_output_million: f64,
    pub timeout_secs: u64,
    pub payload_template: Option<String>,
    pub headers_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealthMetrics {
    pub provider_id: String,
    pub success_count: i32,
    pub failure_count: i32,
    pub consecutive_failures: i32,
    pub average_latency_ms: i32,
    pub health_score: f64,
    pub quality_score: f64,
    pub last_error_type: Option<String>,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn generate(&self, prompt: &str, system_prompt: Option<&str>, max_tokens: Option<usize>) -> Result<String>;
    fn name(&self) -> &str;
    fn api_url(&self) -> &str;
    fn get_capabilities(&self) -> Vec<ModelCapability>;
    fn model_name(&self) -> &str;
    fn id(&self) -> &str;
    fn family(&self) -> &str;
}

pub struct OllamaProvider {
    pub id: String,
    pub name: String,
    pub api_url: String,
    pub model_name: String,
    pub capabilities: Vec<ModelCapability>,
    pub payload_template: Option<String>,
    pub headers_template: Option<String>,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(
        id: String,
        name: String,
        api_url: String,
        model_name: String,
        capabilities: Vec<ModelCapability>,
        payload_template: Option<String>,
        headers_template: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            api_url,
            model_name,
            capabilities,
            payload_template,
            headers_template,
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

        let mut req = self.client.post(&endpoint);

        // Inject custom headers if provided
        if let Some(ref headers_raw) = self.headers_template {
            if let Ok(headers_obj) = serde_json::from_str::<serde_json::Value>(headers_raw) {
                if let Some(obj) = headers_obj.as_object() {
                    for (k, v) in obj {
                        if let Some(val_str) = v.as_str() {
                            req = req.header(k, val_str);
                        }
                    }
                }
            }
        }

        // Handle dynamic payload templates
        let res = if let Some(ref template) = self.payload_template {
            let mut body = template.clone();
            body = body.replace("{{prompt}}", prompt);
            if let Some(sys) = system_prompt {
                body = body.replace("{{system}}", sys);
            } else {
                body = body.replace("{{system}}", "");
            }
            body = body.replace("{{max_tokens}}", &max_tokens.unwrap_or(2048).to_string());
            
            // Check if template is valid JSON
            let parsed_json: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| anyhow!("Invalid custom JSON payload template: {}", e))?;

            req.json(&parsed_json).send().await?
        } else {
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

            req.json(&body).send().await?
        };

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
    fn id(&self) -> &str { &self.id }
    fn family(&self) -> &str { "ollama" }
}

pub struct OpenAiCompatibleProvider {
    pub id: String,
    pub name: String,
    pub api_url: String,
    pub api_key: Option<String>,
    pub model_name: String,
    pub capabilities: Vec<ModelCapability>,
    pub provider_family: String,
    pub payload_template: Option<String>,
    pub headers_template: Option<String>,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        id: String,
        name: String,
        api_url: String,
        api_key: Option<String>,
        model_name: String,
        capabilities: Vec<ModelCapability>,
        provider_family: String,
        payload_template: Option<String>,
        headers_template: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            api_url,
            api_key,
            model_name,
            capabilities,
            provider_family,
            payload_template,
            headers_template,
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

        let mut req = self.client.post(&endpoint);

        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        // Inject custom headers if provided
        if let Some(ref headers_raw) = self.headers_template {
            if let Ok(headers_obj) = serde_json::from_str::<serde_json::Value>(headers_raw) {
                if let Some(obj) = headers_obj.as_object() {
                    for (k, v) in obj {
                        if let Some(val_str) = v.as_str() {
                            req = req.header(k, val_str);
                        }
                    }
                }
            }
        }

        // Handle dynamic payload templates
        let res = if let Some(ref template) = self.payload_template {
            let mut body = template.clone();
            body = body.replace("{{prompt}}", prompt);
            if let Some(sys) = system_prompt {
                body = body.replace("{{system}}", sys);
            } else {
                body = body.replace("{{system}}", "");
            }
            body = body.replace("{{max_tokens}}", &max_tokens.unwrap_or(2048).to_string());

            let parsed_json: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| anyhow!("Invalid custom JSON payload template: {}", e))?;

            req.json(&parsed_json).send().await?
        } else {
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

            req.json(&body).send().await?
        };

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
    fn id(&self) -> &str { &self.id }
    fn family(&self) -> &str { &self.provider_family }
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
            "INSERT INTO provider_configs (
                id, name, api_url, api_key, is_enabled, capabilities, routing_priority, 
                model_name, provider_family, price_input_million, price_output_million, 
                timeout_secs, payload_template, headers_template
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                api_url = excluded.api_url,
                api_key = excluded.api_key,
                is_enabled = excluded.is_enabled,
                capabilities = excluded.capabilities,
                routing_priority = excluded.routing_priority,
                model_name = excluded.model_name,
                provider_family = excluded.provider_family,
                price_input_million = excluded.price_input_million,
                price_output_million = excluded.price_output_million,
                timeout_secs = excluded.timeout_secs,
                payload_template = excluded.payload_template,
                headers_template = excluded.headers_template",
            (
                &provider.id,
                &provider.name,
                &provider.api_url,
                obscured_key,
                if provider.is_enabled { 1 } else { 0 },
                capabilities_json,
                provider.routing_priority,
                &provider.model_name,
                &provider.provider_family,
                provider.price_input_million,
                provider.price_output_million,
                provider.timeout_secs as i64,
                &provider.payload_template,
                &provider.headers_template,
            ),
        )?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn remove_provider(&self, id: &str) -> Result<()> {
        info!(id = %id, "Removing provider from registry");
        let conn = self.storage.conn.lock().unwrap();
        conn.execute("DELETE FROM provider_configs WHERE id = ?1", [id])?;
        conn.execute("DELETE FROM provider_health WHERE provider_id = ?1", [id])?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_providers(&self) -> Result<Vec<RegisteredProvider>> {
        let conn = self.storage.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, api_url, api_key, is_enabled, capabilities, routing_priority, model_name, 
                    provider_family, price_input_million, price_output_million, timeout_secs, 
                    payload_template, headers_template 
             FROM provider_configs"
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
            let provider_family: String = row.get(8)?;
            let price_input_million: f64 = row.get(9)?;
            let price_output_million: f64 = row.get(10)?;
            let timeout_secs: i64 = row.get(11)?;
            let payload_template: Option<String> = row.get(12)?;
            let headers_template: Option<String> = row.get(13)?;

            let capabilities: Vec<ModelCapability> = serde_json::from_str(&capabilities_raw)
                .unwrap_or_else(|_| vec![]);

            let api_key = api_key_obscured.map(|k| {
                k.chars().map(|c| ((c as u32) - 1) as u8 as char).collect::<String>()
            });

            let provider_type = if provider_family == "ollama" {
                "ollama".to_string()
            } else {
                "openai".to_string()
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
                provider_family,
                price_input_million,
                price_output_million,
                timeout_secs: timeout_secs as u64,
                payload_template,
                headers_template,
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

        if p.provider_family == "ollama" {
            Ok(Arc::new(OllamaProvider::new(
                p.id,
                p.name,
                p.api_url,
                p.model_name,
                p.capabilities,
                p.payload_template,
                p.headers_template,
            )))
        } else {
            Ok(Arc::new(OpenAiCompatibleProvider::new(
                p.id,
                p.name,
                p.api_url,
                p.api_key,
                p.model_name,
                p.capabilities,
                p.provider_family,
                p.payload_template,
                p.headers_template,
            )))
        }
    }

    pub fn get_health_metrics(&self, provider_id: &str) -> Result<Option<ProviderHealthMetrics>> {
        let conn = self.storage.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT provider_id, success_count, failure_count, consecutive_failures, 
                    average_latency_ms, health_score, quality_score, last_error_type 
             FROM provider_health WHERE provider_id = ?1"
        )?;

        let mut rows = stmt.query_map([provider_id], |row| {
            let provider_id: String = row.get(0)?;
            let success_count: i32 = row.get(1)?;
            let failure_count: i32 = row.get(2)?;
            let consecutive_failures: i32 = row.get(3)?;
            let average_latency_ms: i32 = row.get(4)?;
            let health_score: f64 = row.get(5)?;
            let quality_score: f64 = row.get(6)?;
            let last_error_type: Option<String> = row.get(7)?;

            Ok(ProviderHealthMetrics {
                provider_id,
                success_count,
                failure_count,
                consecutive_failures,
                average_latency_ms,
                health_score,
                quality_score,
                last_error_type,
            })
        })?;

        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    pub fn update_health_metrics(&self, provider_id: &str, success: bool, latency_ms: u64, error_type: Option<String>) -> Result<()> {
        let conn = self.storage.conn.lock().unwrap();
        
        let current = {
            let stmt = conn.prepare(
                "SELECT provider_id, success_count, failure_count, consecutive_failures, 
                        average_latency_ms, health_score, quality_score, last_error_type 
                 FROM provider_health WHERE provider_id = ?1"
            ).ok();

            stmt.and_then(|mut s| {
                s.query_row([provider_id], |row| {
                    let provider_id: String = row.get(0)?;
                    let success_count: i32 = row.get(1)?;
                    let failure_count: i32 = row.get(2)?;
                    let consecutive_failures: i32 = row.get(3)?;
                    let average_latency_ms: i32 = row.get(4)?;
                    let health_score: f64 = row.get(5)?;
                    let quality_score: f64 = row.get(6)?;
                    let last_error_type: Option<String> = row.get(7)?;

                    Ok(ProviderHealthMetrics {
                        provider_id,
                        success_count,
                        failure_count,
                        consecutive_failures,
                        average_latency_ms,
                        health_score,
                        quality_score,
                        last_error_type,
                    })
                }).ok()
            })
        };

        let (suc, fail, consec, avg_lat, h_score, q_score) = if let Some(c) = current {
            let s = if success { c.success_count + 1 } else { c.success_count };
            let f = if !success { c.failure_count + 1 } else { c.failure_count };
            let consec = if success { 0 } else { c.consecutive_failures + 1 };
            
            let avg_lat = if success {
                if c.success_count > 0 {
                    ((c.average_latency_ms * c.success_count) + latency_ms as i32) / (c.success_count + 1)
                } else {
                    latency_ms as i32
                }
            } else {
                c.average_latency_ms
            };

            let mut h_score = c.health_score;
            if !success {
                h_score = (h_score - 0.15).max(0.0);
            } else {
                h_score = (h_score + 0.05).min(1.0);
            }

            let mut q_score = c.quality_score;
            if !success && error_type.as_deref() == Some("VerificationFailure") {
                q_score = (q_score - 0.20).max(0.0);
            } else if success {
                q_score = (q_score + 0.02).min(1.0);
            }

            (s, f, consec, avg_lat, h_score, q_score)
        } else {
            let s = if success { 1 } else { 0 };
            let f = if !success { 1 } else { 0 };
            let consec = if success { 0 } else { 1 };
            let h_score = if success { 1.0 } else { 0.85 };
            let q_score = 1.0;
            (s, f, consec, latency_ms as i32, h_score, q_score)
        };

        conn.execute(
            "INSERT INTO provider_health (
                provider_id, success_count, failure_count, consecutive_failures, 
                average_latency_ms, health_score, quality_score, last_error_type
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(provider_id) DO UPDATE SET
                success_count = excluded.success_count,
                failure_count = excluded.failure_count,
                consecutive_failures = excluded.consecutive_failures,
                average_latency_ms = excluded.average_latency_ms,
                health_score = excluded.health_score,
                quality_score = excluded.quality_score,
                last_error_type = excluded.last_error_type",
            (
                provider_id,
                suc,
                fail,
                consec,
                avg_lat,
                h_score,
                q_score,
                &error_type,
            ),
        )?;

        Ok(())
    }
}
