use super::*;

#[test]
fn blur_keeps_total_alpha() {
    let mut a = vec![0.0; 41 * 41];
    a[20 * 41 + 20] = 1.0;
    blur::gaussian(&mut a, 41, 41, 3.0);
    let total: f32 = a.iter().sum();
    assert!((total - 1.0).abs() < 0.01, "{total}");
    assert!(a[20 * 41 + 20] < 0.05);
}

#[test]
fn glows_have_content_where_expected() {
    let o = outer(2.0);
    let alpha = |s: &Sprite, x: u32, y: u32| s.rgba[((y * s.width + x) * 4 + 3) as usize];
    // just outside the border, left middle: glow; far corner of the margin: nearly none
    let (bx, by) = (((0.0 - o.left - 1.5) * 2.0) as u32, ((55.0 - o.top) * 2.0) as u32);
    assert!(alpha(&o, bx, by) > 0, "outer glow next to the border");
    assert!(alpha(&o, 0, 0) <= 1);
    let t = title(2.0);
    let peak = t.rgba.chunks(4).map(|p| p[3]).max().unwrap();
    assert!((40..200).contains(&peak), "title glow peak {peak}");
    let i = inner(2.0);
    assert!(alpha(&i, 4, 110) > alpha(&i, 240, 110), "inner glow strongest at the edge");
    assert_eq!(digits(2.0).len(), 10);
    assert_eq!(centered(32.0, "1732"), 32.0 - 2.0 * ADVANCE);
}
