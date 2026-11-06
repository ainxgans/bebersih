use bebersih_lib::models::CleanCategory;
use bebersih_lib::scanner::package::{
    parse_flatpak_unused_output, parse_snap_list_output, scan_snap_and_flatpak_with_config,
    PackageScanConfig,
};

#[test]
fn test_parse_snap_list_output_detects_disabled_revisions() {
    let sample_output = r#"
Name      Version          Rev    Tracking       Publisher     Notes
bare      1.0              5      latest/stable  canonical✓    base
core20    20230801         2015   latest/stable  canonical✓    base,disabled
core20    20240101         2100   latest/stable  canonical✓    base
firefox   120.0-1          3456   latest/stable  mozilla✓      disabled
firefox   122.0-1          3500   latest/stable  mozilla✓      -
"#;

    let items = parse_snap_list_output(sample_output);
    assert_eq!(items.len(), 2);

    let core20 = &items[0];
    assert_eq!(core20.name, "core20");
    assert_eq!(core20.revision, "2015");
    assert_eq!(core20.snap_path, "/var/lib/snapd/snaps/core20_2015.snap");

    let firefox = &items[1];
    assert_eq!(firefox.name, "firefox");
    assert_eq!(firefox.revision, "3456");
    assert_eq!(firefox.snap_path, "/var/lib/snapd/snaps/firefox_3456.snap");
}

#[test]
fn test_parse_flatpak_unused_output() {
    let sample_output = r#"
ID                                                  Branch   Op   Remote   Download
org.freedesktop.Platform.VAAPI.Intel/x86_64/22.08   22.08    u    flathub  12.5 MB
org.gnome.Platform/x86_64/44                        44       u    flathub  350.0 MB
"#;

    let items = parse_flatpak_unused_output(sample_output);
    assert_eq!(items.len(), 2);

    assert_eq!(items[0].id, "org.freedesktop.Platform.VAAPI.Intel/x86_64/22.08");
    assert_eq!(items[0].size_bytes, 12_500_000);

    assert_eq!(items[1].id, "org.gnome.Platform/x86_64/44");
    assert_eq!(items[1].size_bytes, 350_000_000);
}

#[test]
fn test_scan_snap_and_flatpak_with_config() {
    let config = PackageScanConfig {
        snap_output: Some(
            "Name  Version  Rev   Tracking  Publisher  Notes\ngnome-3-38  1.0  140  stable  canonical✓  disabled\n"
                .to_string(),
        ),
        flatpak_output: Some(
            "ID  Branch  Op  Remote  Download\norg.kde.Platform/x86_64/5.15-22.08  5.15-22.08  u  flathub  200.0 MB\n"
                .to_string(),
        ),
        snap_file_sizes: [("/var/lib/snapd/snaps/gnome-3-38_140.snap".to_string(), 250_000_000)]
            .into_iter()
            .collect(),
    };

    let items = scan_snap_and_flatpak_with_config(&config).expect("scan package caches");
    assert_eq!(items.len(), 2);

    let snap_item = items.iter().find(|i| i.id.starts_with("snap:")).expect("snap item found");
    assert_eq!(snap_item.category, CleanCategory::SnapFlatpak);
    assert_eq!(snap_item.size_bytes, 250_000_000);
    assert_eq!(snap_item.requires_root, true);
    assert_eq!(snap_item.supports_trash, false);

    let flatpak_item = items.iter().find(|i| i.id.starts_with("flatpak:")).expect("flatpak item found");
    assert_eq!(flatpak_item.category, CleanCategory::SnapFlatpak);
    assert_eq!(flatpak_item.size_bytes, 200_000_000);
}
