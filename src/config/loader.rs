use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::config::model::{Config, ServerConfig};

/// 設定を読み込む
pub fn load_config(config_path: Option<&str>) -> Result<Config> {
    // JSONファイルからの設定読み込みを試みる
    let mut config = if let Some(path) = config_path {
        load_from_file(path).context("Failed to load config from specified file")?
    } else if let Ok(path) = env::var("CONFIG_FILE") {
        load_from_file(&path).context("Failed to load config from CONFIG_FILE environment variable")?
    } else {
        debug!("No config file specified, using default config");
        Config::default()
    };

    // 環境変数からの設定の上書き
    merge_env_vars(&mut config);

    validate_config(&config)?;
    
    debug!("Loaded config: {:?}", config);
    Ok(config)
}

/// ファイルから設定を読み込む
fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
    info!("Loading config from file: {:?}", path.as_ref());
    let file = File::open(&path)
        .with_context(|| format!("Failed to open config file: {:?}", path.as_ref()))?;
    
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse JSON config from: {:?}", path.as_ref()))?;
    
    Ok(config)
}

/// 環境変数から設定を上書きする
fn merge_env_vars(config: &mut Config) {
    // ホストとポートの環境変数からの設定
    if let Ok(host) = env::var("HOST") {
        config.host = host;
        debug!("Overriding host from environment variable: {}", config.host);
    }
    
    if let Ok(port_str) = env::var("PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            config.port = port;
            debug!("Overriding port from environment variable: {}", config.port);
        } else {
            warn!("Invalid PORT environment variable: {}", port_str);
        }
    }

    // 従来の環境変数からプロセス設定を上書き
    if let Ok(program) = env::var("PROGRAM") {
        let args = env::var("ARGS")
            .unwrap_or_default()
            .split(',')
            .map(String::from)
            .collect::<Vec<_>>();
        
        let mut env_vars = std::collections::HashMap::new();
        for (key, value) in env::vars() {
            if key != "PROGRAM" && key != "ARGS" && key != "HOST" && key != "PORT" && key != "CONFIG_FILE" {
                env_vars.insert(key, value);
            }
        }
        
        // "env" という名前でサーバーを追加
        config.servers.insert("env".to_string(), ServerConfig {
            command: program,
            args,
            env: env_vars,
        });
        
        // デフォルトサーバーが設定されていない場合、環境変数から設定したサーバーをデフォルトに
        if config.default_server.is_none() {
            config.default_server = Some("env".to_string());
            debug!("Setting default server to 'env' from environment variables");
        }
    }
}

/// 設定の妥当性を検証する
fn validate_config(config: &Config) -> Result<()> {
    if config.servers.is_empty() {
        return Err(anyhow::anyhow!("No server configurations found"));
    }
    
    // デフォルトサーバーが指定されていない場合は先頭のサーバーをデフォルトとする
    if config.default_server.is_none() && !config.servers.is_empty() {
        let first_server = config.servers.keys().next().unwrap().clone();
        warn!("No default server specified, using first server: {}", first_server);
    }
    
    // デフォルトサーバーが存在するか確認
    if let Some(ref default_server) = config.default_server {
        if !config.servers.contains_key(default_server) {
            return Err(anyhow::anyhow!(
                "Default server '{}' not found in server configurations",
                default_server
            ));
        }
    }
    
    Ok(())
}