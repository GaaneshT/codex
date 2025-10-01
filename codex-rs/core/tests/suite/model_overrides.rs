use codex_core::BUILT_IN_OSS_MODEL_PROVIDER_ID;
use codex_core::CodexAuth;
use codex_core::ConversationManager;
use codex_core::DEFAULT_OSS_MODEL;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_core::config::ConfigToml;
use codex_core::protocol::EventMsg;
use codex_core::protocol::Op;
use codex_core::protocol_config_types::ReasoningEffort;
use core_test_support::load_default_config_for_test;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use std::sync::LazyLock;
use std::sync::Mutex;
use tempfile::TempDir;

const CONFIG_TOML: &str = "config.toml";

static ENV_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn set_env_var(key: &str, value: &str) {
    // Safety: all call sites hold ENV_MUTEX to serialize environment mutations
    // during the test run.
    unsafe { std::env::set_var(key, value) };
}

fn remove_env_var(key: &str) {
    // Safety: protected by ENV_MUTEX to avoid concurrent environment access.
    unsafe { std::env::remove_var(key) };
}

#[test]
fn default_config_uses_local_oss_provider() {
    let codex_home = TempDir::new().unwrap();
    let config = Config::load_from_base_config_with_overrides(
        ConfigToml::default(),
        ConfigOverrides::default(),
        codex_home.path().to_path_buf(),
    )
    .expect("load default config");

    assert_eq!(config.model_provider_id, BUILT_IN_OSS_MODEL_PROVIDER_ID);
    assert_eq!(config.model, DEFAULT_OSS_MODEL);
}

#[test]
fn env_provider_overrides_default() {
    let codex_home = TempDir::new().unwrap();
    let _guard = ENV_MUTEX.lock().unwrap();
    let prev_provider = std::env::var("CODEX_PROVIDER").ok();
    let prev_api_key = std::env::var("OPENAI_API_KEY").ok();
    remove_env_var("OPENAI_API_KEY");
    set_env_var("CODEX_PROVIDER", "openai");

    let config = Config::load_from_base_config_with_overrides(
        ConfigToml::default(),
        ConfigOverrides::default(),
        codex_home.path().to_path_buf(),
    )
    .expect("load config with env provider");

    assert_eq!(config.model_provider_id, "openai");

    match prev_provider {
        Some(value) => set_env_var("CODEX_PROVIDER", &value),
        None => remove_env_var("CODEX_PROVIDER"),
    }
    match prev_api_key {
        Some(value) => set_env_var("OPENAI_API_KEY", &value),
        None => remove_env_var("OPENAI_API_KEY"),
    }
}

#[test]
fn env_model_overrides_default() {
    let codex_home = TempDir::new().unwrap();
    let _guard = ENV_MUTEX.lock().unwrap();
    let prev_model = std::env::var("CODEX_MODEL").ok();
    set_env_var("CODEX_MODEL", "custom-model");

    let config = Config::load_from_base_config_with_overrides(
        ConfigToml::default(),
        ConfigOverrides::default(),
        codex_home.path().to_path_buf(),
    )
    .expect("load config with env model");

    assert_eq!(config.model, "custom-model");

    match prev_model {
        Some(value) => set_env_var("CODEX_MODEL", &value),
        None => remove_env_var("CODEX_MODEL"),
    }
}

#[test]
fn default_provider_prefers_openai_when_api_key_present() {
    let codex_home = TempDir::new().unwrap();
    let _guard = ENV_MUTEX.lock().unwrap();
    let prev_provider = std::env::var("CODEX_PROVIDER").ok();
    let prev_api_key = std::env::var("OPENAI_API_KEY").ok();
    remove_env_var("CODEX_PROVIDER");
    set_env_var("OPENAI_API_KEY", "sk-test");

    let config = Config::load_from_base_config_with_overrides(
        ConfigToml::default(),
        ConfigOverrides::default(),
        codex_home.path().to_path_buf(),
    )
    .expect("load config with api key");

    assert_eq!(config.model_provider_id, "openai");

    match prev_provider {
        Some(value) => set_env_var("CODEX_PROVIDER", &value),
        None => remove_env_var("CODEX_PROVIDER"),
    }

    match prev_api_key {
        Some(value) => set_env_var("OPENAI_API_KEY", &value),
        None => remove_env_var("OPENAI_API_KEY"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn override_turn_context_does_not_persist_when_config_exists() {
    let codex_home = TempDir::new().unwrap();
    let config_path = codex_home.path().join(CONFIG_TOML);
    let initial_contents = "model = \"gpt-4o\"\n";
    tokio::fs::write(&config_path, initial_contents)
        .await
        .expect("seed config.toml");

    let mut config = load_default_config_for_test(&codex_home);
    config.model = "gpt-4o".to_string();

    let conversation_manager =
        ConversationManager::with_auth(CodexAuth::from_api_key("Test API Key"));
    let codex = conversation_manager
        .new_conversation(config)
        .await
        .expect("create conversation")
        .conversation;

    codex
        .submit(Op::OverrideTurnContext {
            cwd: None,
            approval_policy: None,
            sandbox_policy: None,
            model: Some("o3".to_string()),
            effort: Some(Some(ReasoningEffort::High)),
            summary: None,
        })
        .await
        .expect("submit override");

    codex.submit(Op::Shutdown).await.expect("request shutdown");
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::ShutdownComplete)).await;

    let contents = tokio::fs::read_to_string(&config_path)
        .await
        .expect("read config.toml after override");
    assert_eq!(contents, initial_contents);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn override_turn_context_does_not_create_config_file() {
    let codex_home = TempDir::new().unwrap();
    let config_path = codex_home.path().join(CONFIG_TOML);
    assert!(
        !config_path.exists(),
        "test setup should start without config"
    );

    let config = load_default_config_for_test(&codex_home);

    let conversation_manager =
        ConversationManager::with_auth(CodexAuth::from_api_key("Test API Key"));
    let codex = conversation_manager
        .new_conversation(config)
        .await
        .expect("create conversation")
        .conversation;

    codex
        .submit(Op::OverrideTurnContext {
            cwd: None,
            approval_policy: None,
            sandbox_policy: None,
            model: Some("o3".to_string()),
            effort: Some(Some(ReasoningEffort::Medium)),
            summary: None,
        })
        .await
        .expect("submit override");

    codex.submit(Op::Shutdown).await.expect("request shutdown");
    wait_for_event(&codex, |ev| matches!(ev, EventMsg::ShutdownComplete)).await;

    assert!(
        !config_path.exists(),
        "override should not create config.toml"
    );
}
