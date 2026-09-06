use super::rounded_corner_coverage;

#[test]
fn rounded_corner_coverage_is_symmetric_across_axes() {
    let radius = 8;
    for local in 0..radius {
        let a = rounded_corner_coverage(local, 0, radius);
        let b = rounded_corner_coverage(0, local, radius);
        assert!((a - b).abs() < f32::EPSILON);
    }
}

#[test]
fn rounded_corner_coverage_has_multi_pixel_fringe() {
    let radius = 8;
    let mut partials = 0u32;
    let mut solids = 0u32;
    for ly in 0..radius {
        for lx in 0..radius {
            let coverage = rounded_corner_coverage(lx, ly, radius);
            if coverage > 0.0 && coverage < 1.0 {
                partials += 1;
            } else if coverage >= 1.0 {
                solids += 1;
            }
        }
    }
    assert!(partials > 1, "expected a multi-pixel antialiased fringe");
    assert!(solids > 0, "expected fully covered interior corner pixels");
    // Outer tip should be empty or very soft, not a hard stair.
    assert!(rounded_corner_coverage(0, 0, radius) < 0.5);
}
