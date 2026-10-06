use unibrowser_project::{blit_rgba_with_transform, LayerTransform};

#[test]
fn blit_applies_offset() {
    let src = vec![255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255];
    let mut dst = vec![0u8; 16 * 4];
    let t = LayerTransform::with_offset(1.0, 0.0);
    blit_rgba_with_transform(&mut dst, 4, 4, &src, 2, 2, &t);
    assert_eq!(dst[4], 255);
}
