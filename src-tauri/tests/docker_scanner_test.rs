use bebersih_lib::models::CleanCategory;
use bebersih_lib::scanner::docker::{
    parse_docker_df_output, parse_docker_df_summary, parse_docker_df_verbose, parse_docker_size,
    scan_docker_caches, scan_docker_caches_with_config, DockerScanConfig,
};

#[test]
fn test_parse_docker_size() {
    assert_eq!(parse_docker_size("0B"), Some(0));
    assert_eq!(parse_docker_size("0"), Some(0));
    assert_eq!(parse_docker_size("0.0"), Some(0));
    assert_eq!(parse_docker_size("100B"), Some(100));
    assert_eq!(parse_docker_size("100 bytes"), Some(100));
    assert_eq!(parse_docker_size("1kB"), Some(1024));
    assert_eq!(parse_docker_size("1.5KB"), Some(1536));
    assert_eq!(parse_docker_size("10MB"), Some(10 * 1024 * 1024));
    assert_eq!(parse_docker_size("2.5GB"), Some((2.5 * 1024.0 * 1024.0 * 1024.0) as u64));
    assert_eq!(parse_docker_size("7.22GB (37%)"), Some((7.22 * 1024.0 * 1024.0 * 1024.0) as u64));
    assert_eq!(parse_docker_size("N/A"), None);
    assert_eq!(parse_docker_size(""), None);
    assert_eq!(parse_docker_size("invalid"), None);
}

#[test]
fn test_parse_docker_df_output_summary() {
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
    // CRITICAL SAFETY: Docker volumes must NOT be selected by default
    assert!(!volumes.selected);

    let build_cache = items.iter().find(|i| i.id == "docker:build_cache").unwrap();
    assert_eq!(build_cache.category, CleanCategory::DockerCache);
    assert!(build_cache.selected);
}

#[test]
fn test_parse_docker_df_verbose() {
    let sample_json = r#"{
        "Images": [
            {
                "ID": "sha256:dangling123456789",
                "Repository": "<none>",
                "Tag": "<none>",
                "Size": "250MB"
            },
            {
                "ID": "sha256:activeimage123456",
                "Repository": "my-app",
                "Tag": "latest",
                "Size": "500MB"
            }
        ],
        "Containers": [
            {
                "ID": "cont_stopped_123456",
                "Names": "bench-test-stopped",
                "State": "exited",
                "Status": "Exited (0) 2 days ago",
                "Size": "50MB"
            },
            {
                "ID": "cont_running_123456",
                "Names": "bench-test-running",
                "State": "running",
                "Status": "Up 2 hours",
                "Size": "100MB"
            }
        ],
        "Volumes": [
            {
                "Name": "unused_vol_1",
                "Links": "0",
                "Size": "80MB",
                "Mountpoint": "/var/lib/docker/volumes/unused_vol_1/_data"
            },
            {
                "Name": "active_vol_1",
                "Links": "1",
                "Size": "120MB",
                "Mountpoint": "/var/lib/docker/volumes/active_vol_1/_data"
            }
        ],
        "BuildCache": [
            {
                "ID": "cache_build_123",
                "Description": "buildkit cache",
                "Size": "300MB",
                "InUse": false,
                "Reclaimable": "true"
            },
            {
                "ID": "cache_build_active",
                "Description": "in-use cache",
                "Size": "400MB",
                "InUse": true,
                "Reclaimable": "false"
            }
        ]
    }"#;

    let items = parse_docker_df_verbose(sample_json).expect("should parse valid docker df verbose");
    assert_eq!(items.len(), 4);

    let dangling_img = items
        .iter()
        .find(|item| item.id.starts_with("docker:image:"))
        .expect("dangling image missing");
    assert_eq!(dangling_img.category, CleanCategory::DockerCache);
    assert_eq!(dangling_img.size_bytes, 250 * 1024 * 1024);
    assert!(dangling_img.selected);

    let stopped_cont = items
        .iter()
        .find(|item| item.id.starts_with("docker:container:"))
        .expect("stopped container missing");
    assert_eq!(stopped_cont.category, CleanCategory::DockerCache);
    assert!(stopped_cont.selected);
    assert!(stopped_cont.label.contains("bench-test-stopped"));

    let unused_vol = items
        .iter()
        .find(|item| item.id.starts_with("docker:volume:"))
        .expect("unused volume missing");
    assert_eq!(unused_vol.category, CleanCategory::DockerVolume);
    // CRITICAL SAFETY REQUIREMENT: Docker volumes must NOT be selected by default
    assert!(!unused_vol.selected);
    assert_eq!(
        unused_vol.path,
        Some("/var/lib/docker/volumes/unused_vol_1/_data".to_string())
    );

    let build_cache = items
        .iter()
        .find(|item| item.id.starts_with("docker:buildcache:"))
        .expect("build cache missing");
    assert_eq!(build_cache.category, CleanCategory::DockerCache);
    assert!(build_cache.selected);
    assert!(build_cache.label.contains("buildkit cache"));

    // Also verify parse_docker_df_output automatically routes verbose JSON to verbose parser
    let routed_items = parse_docker_df_output(sample_json).expect("auto route verbose");
    assert_eq!(routed_items.len(), 4);
}

#[test]
fn test_parse_docker_df_summary_explicit() {
    let sample = r#"
{"Active":"0","Reclaimable":"100MB (100%)","Size":"100MB","TotalCount":"1","Type":"Containers"}
"#;
    let items = parse_docker_df_summary(sample).expect("parse summary");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "docker:containers");
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

#[test]
fn test_scan_docker_caches_graceful() {
    let res = scan_docker_caches();
    assert!(res.is_ok(), "scan_docker_caches should never panic or error out");
}
