
use super::{
    centered_rect, font_data_matches_name, font_match_sort_key, horizontal_pane_rects,
    horizontal_pane_rects_for_active, normalize_font_name, preferred_font_search_roots, rect_tuple,
    vertical_pane_rects, vertical_pane_rects_for_active,
};
use std::path::Path;

#[test]
fn centered_rect_places_content_in_middle() {
    assert_eq!(rect_tuple(centered_rect(100, 80, 40, 20)), (30, 30, 40, 20));
}

#[test]
fn horizontal_split_returns_two_stacked_rects() {
    let rects = horizontal_pane_rects(120, 60, 2);
    assert_eq!(rect_tuple(rects[0]), (0, 0, 120, 30));
    assert_eq!(rect_tuple(rects[1]), (0, 30, 120, 30));
}

#[test]
fn font_metadata_matching_accepts_family_names() {
    let request = normalize_font_name("Material Icons");
    assert_ne!(request, normalize_font_name("material-design-icons"));
    let font_data = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../volt/assets/font/material-design-icons.ttf"
    ));
    assert!(font_data_matches_name(font_data, &request));
    assert!(font_data_matches_name(
        font_data,
        &normalize_font_name("MaterialIcons-Regular")
    ));
}

#[test]
fn font_match_sort_key_prefers_regular_faces_for_family_requests() {
    let normalized = normalize_font_name("Liga Berkeley Mono");
    assert!(
        font_match_sort_key(Path::new("LigaBerkeleyMono-Regular.ttf"), &normalized,)
            < font_match_sort_key(Path::new("LigaBerkeleyMono-Bold.ttf"), &normalized,)
    );
    assert!(
        font_match_sort_key(Path::new("LigaBerkeleyMono-Regular.ttf"), &normalized,)
            < font_match_sort_key(Path::new("Berkeley Mono Variable.ttf"), &normalized,)
    );
}

#[test]
fn vertical_split_returns_two_side_by_side_rects() {
    let rects = vertical_pane_rects(120, 60, 2);
    assert_eq!(rect_tuple(rects[0]), (0, 0, 60, 60));
    assert_eq!(rect_tuple(rects[1]), (60, 0, 60, 60));
}

#[test]
fn vertical_golden_ratio_grows_the_first_active_pane() {
    let rects = vertical_pane_rects_for_active(160, 120, 2, 0, true);
    assert_eq!(rect_tuple(rects[0]), (0, 0, 99, 120));
    assert_eq!(rect_tuple(rects[1]), (99, 0, 61, 120));
}

#[test]
fn vertical_golden_ratio_grows_the_second_active_pane() {
    let rects = vertical_pane_rects_for_active(160, 120, 2, 1, true);
    assert_eq!(rect_tuple(rects[0]), (0, 0, 61, 120));
    assert_eq!(rect_tuple(rects[1]), (61, 0, 99, 120));
}

#[test]
fn horizontal_golden_ratio_grows_the_first_active_pane() {
    let rects = horizontal_pane_rects_for_active(200, 100, 2, 0, true);
    assert_eq!(rect_tuple(rects[0]), (0, 0, 200, 62));
    assert_eq!(rect_tuple(rects[1]), (0, 62, 200, 38));
}

#[test]
fn horizontal_golden_ratio_grows_the_second_active_pane() {
    let rects = horizontal_pane_rects_for_active(200, 100, 2, 1, true);
    assert_eq!(rect_tuple(rects[0]), (0, 0, 200, 38));
    assert_eq!(rect_tuple(rects[1]), (0, 38, 200, 62));
}

#[test]
fn font_search_roots_include_platform_locations() {
    assert!(!preferred_font_search_roots().is_empty());
}
