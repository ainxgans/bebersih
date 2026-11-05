use bebersih_lib::models::CleanCategory;
use bebersih_lib::scanner::docker::{
    parse_docker_df_output, parse_docker_size, scan_docker_caches_with_config, DockerScanConfig,
};

#[test]
fn test_parse_docker_size() {
    assert_eq!(parse_docker_size("0B"), Some(0));
    assert_eq!(parse_docker_size("100B"), Some(100));
    assert_eq!(parse_docker_size("1kB"), Some(1024));
    assert_eq!(parse_docker_size("1.5KB"), Some(1536));
    assert_eq!(parse_docker_size("10MB"), Some(10 * 1024 * 1024));
    assert_eq!(parse_docker_size("2.5GB"), Some((2.5 * 1024.0 * 1024.0 * 1024.0) as u64));
    assert_eq!(parse_docker_size("7.22GB (37%)"), Some((7.22 * 1024.0 * 1024.0 * 1024.0) as u64));
    assert_eq!(parse_docker_size("invalid"), None);
}

#[test]
fn test_parse_docker_df_output() {
    let sample = r#"
{"Active":"41","Reclaimable":"7.22GB (37%)","Size":"19.06GB","TotalCount":"122","Type":"Images"}
{"Active":"15","Reclaimable":"2.431GB (92%)","Size":"2.63GB","TotalCount":"47","Type":"Containers"}
{"Active":"16","Reclaimable":"2.534GB (54%)","Size":"4.642GB","TotalCount":"40","Type":"Local Volumes"}
{"Active":"0","Reclaimable":"500MB","Size":"500MB","TotalCount":"10","Type":"Build Cache"}
"#;

    let items = parse_docker_df_output(sample).expect("should parse docker output");
    assert_eq!(items.len(), 4);

    let images = items.iter().find(|i| i.id == "docker:images").unwrap();
    assert_eq!(images.category, CleanCategory::DockerCache);
    assert!(images.selected);
    assert!(!images.supports_trash);
    assert!(!images.requires_root);

    let containers = items.iter().find(|i| i.id == "docker:containers").unwrap();
    assert_eq!(containers.category, CleanCategory::DockerCache);
    assert!(containers.selected);

    let volumes = items.iter().find(|i| i.id == "docker:volumes").unwrap();
    assert_eq!(volumes.category, CleanCategory::DockerVolume);
    // CRITICAL: Docker volumes must NOT be selected by default
    assert!(!volumes.selected);

    let build_cache = items.iter().find(|i| i.id == "docker:build_cache").unwrap();
    assert_eq!(build_cache.category, CleanCategory::DockerCache);
    assert!(build_cache.selected);
}

#[test]
fn test_scan_docker_caches_with_config() {
    let sample = r#"
{"Active":"0","Reclaimable":"0B","Size":"0B","TotalCount":"0","Type":"Images"}
{"Active":"0","Reclaimable":"100MB (100%)","Size":"100MB","TotalCount":"1","Type":"Containers"}
"#;
    let config = DockerScanConfig {
        custom_output: Some(sample.to_string()),
        force_disabled: false,
    };

    let items = scan_docker_caches_with_config(&config).unwrap();
    // 0B items are omitted
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "docker:containers");
}

#[test]
fn test_scan_docker_caches_disabled_or_missing() {
    let config = DockerScanConfig {
        custom_output: None,
        force_disabled: true,
    };

    let items = scan_docker_caches_with_config(&config).unwrap();
    assert!(items.is_empty());
}
