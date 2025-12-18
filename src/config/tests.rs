#[cfg(test)]
mod tests {
    use crate::config::types::*;
    use std::path::PathBuf;

    #[test]
    fn test_config_serialization() {
        let config = CudaMgrConfig {
            install_dir: PathBuf::from("/opt/cudamgr"),
            cache_dir: PathBuf::from("/tmp/cudamgr"),
            log_level: LogLevel::Debug,
            auto_cleanup: false,
            verify_downloads: true,
            parallel_downloads: true,
            max_concurrent_downloads: 5,
            default_cuda_version: Some("11.8".to_string()),
            proxy_settings: Some(ProxyConfig {
                http_proxy: Some("http://proxy:8080".to_string()),
                https_proxy: Some("https://proxy:8080".to_string()),
                no_proxy: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            }),
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: CudaMgrConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
        assert_eq!(config.max_concurrent_downloads, 5);
        assert!(config.proxy_settings.is_some());
    }

    #[test]
    fn test_config_default() {
        let config = CudaMgrConfig::default();

        assert!(config.install_dir.to_string_lossy().contains(".cudamgr"));
        assert!(config.cache_dir.to_string_lossy().contains("cache"));
        assert_eq!(config.max_concurrent_downloads, 3);
        assert!(config.auto_cleanup);
        assert!(config.verify_downloads);
        assert!(config.parallel_downloads);
        assert!(config.proxy_settings.is_none());
    }

    #[test]
    fn test_log_level_serialization() {
        let levels = vec![
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: LogLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(level, deserialized);
        }
    }

    #[test]
    fn test_proxy_config_serialization() {
        let proxy = ProxyConfig {
            http_proxy: Some("http://proxy.example.com:8080".to_string()),
            https_proxy: None,
            no_proxy: vec!["localhost".to_string(), "*.local".to_string()],
        };

        let json = serde_json::to_string(&proxy).unwrap();
        let deserialized: ProxyConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(proxy, deserialized);
        assert_eq!(proxy.no_proxy.len(), 2);
    }
}
