use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// サーバー設定全体を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 利用可能なサーバー設定のマップ
    pub servers: HashMap<String, ServerConfig>,
    /// デフォルトで使用するサーバー名
    #[serde(default)]
    pub default_server: Option<String>,
    /// WebSocketサーバーのホスト（デフォルト: "0.0.0.0"）
    #[serde(default = "default_host")]
    pub host: String,
    /// WebSocketサーバーのポート（デフォルト: 8080）
    #[serde(default = "default_port")]
    pub port: u16,
}

/// 個別のサーバー設定を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 実行するコマンド
    pub command: String,
    /// コマンドに渡す引数
    #[serde(default)]
    pub args: Vec<String>,
    /// プロセスに渡す環境変数
    #[serde(default)]
    pub env: HashMap<String, String>,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

impl Default for Config {
    fn default() -> Self {
        Self {
            servers: HashMap::new(),
            default_server: None,
            host: default_host(),
            port: default_port(),
        }
    }
}

/// 設定ファイルの例
pub fn example_config() -> Config {
    let mut servers = HashMap::new();
    
    // filesystem サーバーの設定
    let filesystem_server = ServerConfig {
        command: "npx".to_string(),
        args: vec![
            "-y".to_string(),
            "@modelcontextprotocol/server-filesystem".to_string(),
            "/Users/yonaka/workspace".to_string(),
            "/Users/yonaka/mcp-servers".to_string(),
        ],
        env: HashMap::new(),
    };
    
    // github サーバーの設定
    let mut github_server = ServerConfig {
        command: "npx".to_string(),
        args: vec![
            "-y".to_string(),
            "@modelcontextprotocol/server-github".to_string(),
        ],
        env: HashMap::new(),
    };
    github_server.env.insert("GITHUB_PERSONAL_ACCESS_TOKEN".to_string(), "token_value".to_string());
    
    servers.insert("filesystem".to_string(), filesystem_server);
    servers.insert("github".to_string(), github_server);
    
    Config {
        servers,
        default_server: Some("filesystem".to_string()),
        host: "0.0.0.0".to_string(),
        port: 8080,
    }
}