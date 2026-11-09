//! Unit-level checks of the Hipparcos-derived catalogue (moved from
//! `xalen-stars`, which no longer carries this data).

use xalen_stars_hip_data::{
    GENERATED_CATALOG, GENERATED_STAR_COUNT, GeneratedStarExt, find_conjunctions_expanded,
    find_generated_by_hip, find_generated_by_name,
};

#[test]
fn generated_catalog_count_matches_constant() {
    assert_eq!(GENERATED_CATALOG.len(), GENERATED_STAR_COUNT);
    // The real hip_main.dat (118,218 records) yields 8,870 stars at Vmag<=6.5.
    assert_eq!(GENERATED_CATALOG.len(), 8870);
    assert!(GENERATED_CATALOG.len() >= 6000);
}

#[test]
fn generated_catalog_all_within_vmag_limit() {
    for s in GENERATED_CATALOG {
        assert!(
            s.magnitude <= 6.5,
            "HIP {} magnitude {} exceeds Vmag<=6.5 filter",
            s.hip,
            s.magnitude
        );
    }
}

#[test]
fn generated_catalog_sorted_by_hip_no_dupes() {
    for w in GENERATED_CATALOG.windows(2) {
        assert!(
            w[0].hip < w[1].hip,
            "GENERATED_CATALOG must be strictly sorted by HIP; {} !< {}",
            w[0].hip,
            w[1].hip
        );
    }
}

#[test]
fn generated_positions_are_finite_and_in_range() {
    for s in GENERATED_CATALOG {
        assert!((0.0..360.0).contains(&s.longitude_j2000), "HIP {}", s.hip);
        assert!((-90.0..=90.0).contains(&s.latitude_j2000), "HIP {}", s.hip);
        assert!(s.pm_lon_mas_per_year.is_finite());
        assert!(s.pm_lat_mas_per_year.is_finite());
    }
}

#[test]
fn generated_named_bright_stars_present_by_hip() {
    let aldebaran = find_generated_by_hip(21421).expect("HIP 21421 Aldebaran");
    assert_eq!(aldebaran.name, Some("Aldebaran"));
    let sirius = find_generated_by_hip(32349).expect("HIP 32349 Sirius");
    assert_eq!(sirius.name, Some("Sirius"));
    let polaris = find_generated_by_hip(11767).expect("HIP 11767 Polaris");
    assert_eq!(polaris.name, Some("Polaris"));
    // The name->HIP join did NOT collide with "Polaris Australis".
    assert!(polaris.latitude_j2000 > 60.0);
    assert!(find_generated_by_hip(0).is_none());
}

#[test]
fn generated_find_by_name_case_insensitive() {
    assert!(find_generated_by_name("vega").is_some());
    assert!(find_generated_by_name("VEGA").is_some());
    assert!(find_generated_by_name("definitely-not-a-star").is_none());
}

#[test]
fn formerly_fallback_names_resolve_to_generated_catalog() {
    // Curated spellings that differ from the IAU-WGSN name (or have none):
    // (curated name, expected HIP, expected ecliptic longitude +/- 0.5 deg).
    let cases = [
        ("Zuben Elgenubi", 72622_u32, 225.08),
        ("Zuben Eschamali", 74785, 229.37),
        ("Bharani 41", 13209, 48.2),
        ("Lambda Orionis", 26207, 83.71),
        ("Hyadum II", 20455, 66.87),
        ("Al Jabhah", 49583, 147.9),
    ];
    for (name, hip, lon) in cases {
        let g = find_generated_by_hip(hip).unwrap_or_else(|| panic!("HIP {hip} missing"));
        assert_eq!(g.name, Some(name));
        assert!((g.longitude_j2000 - lon).abs() < 0.5, "{name}");
        assert!(g.pm_lon_mas_per_year.abs() + g.pm_lat_mas_per_year.abs() > 1.0);
    }
}

#[test]
fn expanded_conjunction_finds_stars() {
    let found = find_conjunctions_expanded(69.79, 1.0, 2000.0);
    assert!(found.iter().any(|(s, _)| s.name == Some("Aldebaran")));
    for (_, d) in &found {
        assert!(*d <= 1.0);
    }
}

#[test]
fn extension_trait_methods_agree() {
    let sirius = find_generated_by_hip(32349).unwrap();
    let (lon, lat) = sirius.ecliptic_at_epoch(2100.0);
    assert_eq!(lon, sirius.longitude_at_epoch(2100.0));
    assert_eq!(lat, sirius.latitude_at_epoch(2100.0));
    let jd = 2_451_545.0 + 100.0 * 365.25;
    assert!((sirius.longitude_at_jd(jd) - lon).abs() < 1e-9);
    let fixed = sirius.to_fixed_star("Sirius");
    assert_eq!(fixed.longitude_at_epoch(2100.0), lon);
    // Precession (1.397 deg) plus proper motion (-0.015 deg) in a century.
    assert!((lon - sirius.longitude_j2000 - 1.382).abs() < 0.01);
}
