//! SVG 图形与逐元素表现合同；不是把描边/填充合并成一个新绘制路径。
use crate::*;
#[test]
fn svg_preserves_element_fill_alpha_and_original_widths() {
    let (_,shapes)=icon_svg::parse(r#"<svg viewBox="0 0 24 24" fill="currentColor"><rect x="3" y="3" width="8" height="8" fill-opacity="0.35"/><rect x="13" y="3" width="8" height="8"/></svg>"#).unwrap();
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes[0].style.fill, Some(0.35));
    assert_eq!(shapes[1].style.fill, Some(1.0));
    assert!(shapes[0].style.stroke.is_none());
    let (_,shapes)=icon_svg::parse(r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.35" stroke-linecap="round" stroke-linejoin="round"><path d="M0 0h10"/><path d="M0 0h10v10Z" fill="currentColor" stroke="none"/></svg>"#).unwrap();
    assert_eq!(shapes[0].style.width, 1.35);
    assert!(shapes[0].style.round_cap && shapes[0].style.round_join);
    assert!(shapes[0].style.fill.is_none());
    assert!(shapes[1].style.stroke.is_none());
    assert_eq!(shapes[1].style.fill, Some(1.0));
}
#[test]
fn unsupported_external_or_malformed_svg_is_rejected_without_partial_drawing() {
    for source in [
        r#"<svg viewBox="0 0 24 24"><image href="file:///test"/></svg>"#,
        r#"<svg viewBox="0 0 24 24"><script/></svg>"#,
        r#"<svg viewBox="0 0 24 24"><path d="M0 0" onclick="run()"/></svg>"#,
        r#"<svg viewBox="0 0 24 24"><rect width="NaN" height="8"/></svg>"#,
        r##"<svg viewBox="0 0 24 24" fill="#xyzxyz"><path d="M0 0h10v10Z"/></svg>"##,
        r#"<svg viewBox="0 0 24 24" fill="url(file:///paint)"><path d="M0 0h10v10Z"/></svg>"#,
    ] {
        assert!(icon_svg::parse(source).is_err());
    }
    Icon::svg(r#"<svg viewBox="0 0 16 16" fill="none" stroke="currentColor"><rect x="3" y="3" width="10" height="10" rx="1.2"/><circle cx="8" cy="8" r="3"/><ellipse cx="8" cy="8" rx="3" ry="2"/></svg>"#).validate().unwrap();
}

#[test]
fn static_rgb_is_inherited_per_element_and_current_color_restores_caller_ink() {
    let (_, shapes) = icon_svg::parse(r##"<svg viewBox="0 0 24 24" fill="#06b6d4" stroke="#CBd5E1"><path d="M0 0h10v10Z" fill-opacity="0.5"/><path d="M0 0h8v8Z" fill="currentColor" stroke="none"/><path d="M0 0h6v6Z" fill="#22c55e" stroke="currentColor"/></svg>"##).unwrap();
    assert_eq!(shapes[0].style.fill_color, Some(0x06b6d4));
    assert_eq!(shapes[0].style.stroke_color, Some(0xcbd5e1));
    assert_eq!(shapes[0].style.fill, Some(0.5));
    assert_eq!(shapes[1].style.fill_color, None);
    assert_eq!(shapes[1].style.fill, Some(1.0));
    assert_eq!(shapes[1].style.stroke, None);
    assert_eq!(shapes[2].style.fill_color, Some(0x22c55e));
    assert_eq!(shapes[2].style.stroke_color, None);
    assert_eq!(shapes[2].style.stroke, Some(1.0));
}
