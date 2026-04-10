#[cfg(feature = "toxiproxy")]
use crate::stress::backend::*;

#[cfg(feature = "toxiproxy")]
pub struct ToxiproxyBackend {
    api_url: String,
    proxy_name: String,
    toxic_names: Vec<String>,
}

#[cfg(feature = "toxiproxy")]
impl ToxiproxyBackend {
    pub fn new(api_url: &str, proxy_name: &str) -> Self {
        Self {
            api_url: api_url.trim_end_matches('/').to_string(),
            proxy_name: proxy_name.to_string(),
            toxic_names: Vec::new(),
        }
    }

    fn api_post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
        ureq::post(&format!("{}{}", self.api_url, path))
            .set("Content-Type", "application/json")
            .send_json(body.clone())
            .map_err(|e| format!("Toxiproxy API error: {e}"))
            .and_then(|resp| {
                resp.into_json::<serde_json::Value>()
                    .map_err(|e| format!("JSON parse error: {e}"))
            })
    }

    fn api_delete(&self, path: &str) -> Result<(), String> {
        ureq::delete(&format!("{}{}", self.api_url, path))
            .call()
            .map_err(|e| format!("Toxiproxy API error: {e}"))?;
        Ok(())
    }
}

#[cfg(feature = "toxiproxy")]
impl StressBackend for ToxiproxyBackend {
    fn apply_network_degradation(&mut self, config: &NetworkDegradationConfig) -> Result<(), String> {
        self.remove_network_degradation(&config.interface)?;
        let body = serde_json::json!({
            "name": "stress_latency",
            "type": "latency",
            "stream": "downstream",
            "toxicity": 1.0,
            "attributes": {
                "latency": config.latency_ms as i64,
                "jitter": config.jitter_ms as i64,
            }
        });
        self.api_post(&format!("/proxies/{}/toxics", self.proxy_name), &body)?;
        self.toxic_names.push("stress_latency".to_string());
        Ok(())
    }

    fn remove_network_degradation(&mut self, _interface: &str) -> Result<(), String> {
        for name in self.toxic_names.drain(..) {
            let _ = self.api_delete(&format!("/proxies/{}/toxics/{}", self.proxy_name, name));
        }
        Ok(())
    }

    fn apply_resource_pressure(&mut self, _config: &ResourcePressureConfig) -> Result<(), String> {
        Err("ToxiproxyBackend only handles network faults".into())
    }

    fn remove_resource_pressure(&mut self, _cgroup_path: &str) -> Result<(), String> {
        Ok(())
    }

    fn inject_memory_stress(&mut self, _config: &MemoryStressConfig) -> Result<(), String> {
        Err("ToxiproxyBackend only handles network faults".into())
    }

    fn remove_memory_stress(&mut self) -> Result<(), String> { Ok(()) }

    fn pause_workload(&mut self, _pid: u32) -> Result<(), String> {
        Err("ToxiproxyBackend only handles network faults".into())
    }

    fn resume_workload(&mut self, _pid: u32) -> Result<(), String> { Ok(()) }

    fn apply_network_partition(&mut self, _interface: &str) -> Result<(), String> {
        self.api_post(
            &format!("/proxies/{}", self.proxy_name),
            &serde_json::json!({"enabled": false}),
        )?;
        Ok(())
    }

    fn remove_network_partition(&mut self, _interface: &str) -> Result<(), String> {
        self.api_post(
            &format!("/proxies/{}", self.proxy_name),
            &serde_json::json!({"enabled": true}),
        )?;
        Ok(())
    }

    fn name(&self) -> &str { "toxiproxy" }
}
