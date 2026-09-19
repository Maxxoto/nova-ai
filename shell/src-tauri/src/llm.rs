//! LLM config (RFC-0007): user-supplied OpenAI-compatible endpoint via the
//! Python brain's LiteLLM adapter. Non-secret parts persist in settings.json;
//! the API key lives in the macOS Keychain and is injected as env vars when
//! the brain sidecar spawns — never written to disk in plaintext.

use keyring::Entry;
use serde_json::Value;

use crate::brain::BrainLink;
use crate::settings::LlmSettings;

const KEYCHAIN_SERVICE: &str = "com.ruoxi.shell";
const KEYCHAIN_USER: &str = "llm-api-key";
pub const ENV_BASE_URL: &str = "RUOXI_LLM_BASE_URL";
pub const ENV_API_KEY: &str = "RUOXI_LLM_API_KEY";
pub const ENV_MODEL: &str = "RUOXI_LLM_MODEL";
pub const ENV_VISION_MODEL: &str = "RUOXI_LLM_VISION_MODEL";

fn keychain_entry() -> Result<Entry, String> {
    Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_USER).map_err(|e| format!("keychain unavailable: {e}"))
}

pub fn stored_api_key() -> Option<String> {
    keychain_entry().ok()?.get_password().ok()
}

/// Env vars injected at sidecar spawn; empty when unconfigured.
pub fn env_for_sidecar(settings: &crate::settings::Settings) -> Vec<(String, String)> {
    let mut env = Vec::new();
    if !settings.llm.base_url.is_empty() {
        env.push((ENV_BASE_URL.to_string(), settings.llm.base_url.clone()));
    }
    if let Some(key) = stored_api_key() {
        env.push((ENV_API_KEY.to_string(), key));
    }
    if !settings.llm.model.is_empty() {
        env.push((ENV_MODEL.to_string(), settings.llm.model.clone()));
    }
    if !settings.llm.vision_model.is_empty() {
        env.push((
            ENV_VISION_MODEL.to_string(),
            settings.llm.vision_model.clone(),
        ));
    }
    env
}

#[tauri::command]
pub fn llm_save_config(
    app: tauri::AppHandle,
    base_url: String,
    model: String,
    vision_model: String,
    api_key: Option<String>,
) -> Result<LlmSettings, String> {
    if let Some(key) = api_key
        .as_deref()
        .map(str::trim)
        .filter(|k| !k.is_empty())
    {
        keychain_entry()?.set_password(key).map_err(|e| format!("keychain write failed: {e}"))?;
    }
    let mut current = crate::settings::load(&app);
    let had_key = current.llm.api_key_set || api_key.as_deref().map(str::trim).filter(|k| !k.is_empty()).is_some();
    current.llm = LlmSettings {
        base_url: base_url.trim().to_string(),
        model: model.trim().to_string(),
        vision_model: vision_model.trim().to_string(),
        api_key_set: had_key,
    };
    crate::settings::save(&app, &current)?;
    Ok(current.llm)
}

#[tauri::command]
pub fn llm_clear_api_key(app: tauri::AppHandle) -> Result<LlmSettings, String> {
    keychain_entry()?
        .delete_credential()
        .or_else(|e| match e {
            keyring::Error::NoEntry => Ok(()),
            other => Err(other),
        })
        .map_err(|e| format!("keychain delete failed: {e}"))?;
    let mut current = crate::settings::load(&app);
    current.llm.api_key_set = false;
    crate::settings::save(&app, &current)?;
    Ok(current.llm)
}

/// Round-trips `config.test` through the brain, which performs a 1-token
/// LiteLLM call. Explicit params (freshly typed in the UI) override the
/// spawn-time env config, so a new key can be verified before saving.
#[tauri::command]
pub fn llm_test(
    brain: tauri::State<BrainLink>,
    base_url: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
) -> Result<Value, String> {
    brain.rpc(
        "config.test",
        serde_json::json!({
            "base_url": base_url.unwrap_or_default(),
            "model": model.unwrap_or_default(),
            "api_key": api_key.unwrap_or_default(),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_is_empty_until_configured() {
        let settings = crate::settings::Settings::default();
        assert!(env_for_sidecar(&settings).is_empty());
    }
}
