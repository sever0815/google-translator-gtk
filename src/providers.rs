use std::time::Duration;

pub struct Translation {
    pub text: String,
    pub detected: Option<String>,
}

/// A translation backend. To add a new service, implement this trait
/// and wire the provider up in `app::build_app`.
pub trait Provider: Send + Sync {
    fn translate(&self, text: &str, from: &str, to: &str) -> Result<Translation, String>;
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) translator-gtk/0.1")
        .build()
}

// ---------------------------------------------------------------- Google

pub struct GoogleProvider;

impl GoogleProvider {
    fn call(text: &str, from: &str, to: &str) -> Result<serde_json::Value, String> {
        agent()
            .post("https://translate.googleapis.com/translate_a/single")
            .query("client", "gtx")
            .query("sl", from)
            .query("tl", to)
            .query("dt", "t")
            .send_form(&[("q", text)])
            .map_err(|e| format!("Google request failed: {e}"))?
            .into_json()
            .map_err(|e| format!("Google: bad response: {e}"))
    }
}

impl Provider for GoogleProvider {
    fn translate(&self, text: &str, from: &str, to: &str) -> Result<Translation, String> {
        let v = Self::call(text, from, to)?;
        let mut out = String::new();
        if let Some(segments) = v.get(0).and_then(|s| s.as_array()) {
            for seg in segments {
                if let Some(part) = seg.get(0).and_then(|p| p.as_str()) {
                    out.push_str(part);
                }
            }
        }
        if out.is_empty() {
            return Err("Google: empty translation".into());
        }
        let detected = v.get(2).and_then(|d| d.as_str()).map(str::to_string);
        Ok(Translation { text: out, detected })
    }
}
