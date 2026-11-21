//! 双数值比例组的真实消息、步进、草稿及父销毁回归，不操作外部窗口。
use super::*;
fn fields(value: &str, active: bool) -> Vec<PropertyField> {
    vec![PropertyField {
        id: 7,
        label: "宽高比",
        icon: None,
        kind: PropertyKind::Ratio {
            value: value.into(),
            active,
        },
    }]
}
unsafe fn text(h: HWND) -> String {
    let mut buf = [0u16; 65];
    let n = GetWindowTextW(h, &mut buf) as usize;
    String::from_utf16_lossy(&buf[..n])
}
#[test]
fn paired_ratio_values_keep_colon_separate_and_share_bounded_commit_context() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        set_properties(bar.0, 10, fields("4:3", false), true).unwrap();
        let popup = property_window(bar.0).unwrap();
        let width = GetDlgItem(Some(popup), 0x8000 + 7 * 8).unwrap();
        let height = GetDlgItem(Some(popup), 0x8000 + 7 * 8 + 1).unwrap();
        assert_eq!((text(width), text(height)), ("4".into(), "3".into()));
        SetWindowTextW(width, w!("16")).unwrap();
        set_properties(bar.0, 10, fields("4:3", false), true).unwrap();
        assert_eq!(text(width), "16");
        SendMessageW(height, WM_KEYDOWN, Some(WPARAM(VK_UP.0 as usize)), None);
        let edit = take_property_edit(bar.0, 7).unwrap();
        assert_eq!(edit.context, 10);
        assert_eq!(edit.value.as_deref(), Some("16:4"));
        assert_eq!(edit.active, Some(true));
        SetWindowTextW(width, w!("999")).unwrap();
        SendMessageW(width, WM_KEYDOWN, Some(WPARAM(VK_UP.0 as usize)), None);
        assert_eq!(
            take_property_edit(bar.0, 7).unwrap().value.as_deref(),
            Some("999:4")
        );
        SetWindowTextW(height, w!("1")).unwrap();
        SendMessageW(height, WM_KEYDOWN, Some(WPARAM(VK_DOWN.0 as usize)), None);
        assert_eq!(
            take_property_edit(bar.0, 7).unwrap().value.as_deref(),
            Some("999:1")
        );
        SetWindowTextW(width, w!("0")).unwrap();
        SendMessageW(width, WM_KEYDOWN, Some(WPARAM(VK_RETURN.0 as usize)), None);
        assert!(take_property_edit(bar.0, 7).is_none());
        assert_eq!(text(width), "999");
        SetWindowTextW(width, w!("25")).unwrap();
        SendMessageW(width, WM_KEYDOWN, Some(WPARAM(VK_ESCAPE.0 as usize)), None);
        assert!(take_property_edit(bar.0, 7).is_none());
        assert_eq!(text(width), "999");
        set_properties(bar.0, 11, fields("1:2", true), true).unwrap();
        assert_eq!(GetDlgItem(Some(popup), 0x8000 + 7 * 8).unwrap(), width);
        assert_eq!((text(width), text(height)), ("1".into(), "2".into()));
        let up = GetDlgItem(Some(popup), 0x8000 + 7 * 8 + 2).unwrap();
        SendMessageW(up, BM_CLICK, None, None);
        assert_eq!(
            take_property_edit(bar.0, 7).unwrap().value.as_deref(),
            Some("2:2")
        );
        SendMessageW(
            GetDlgItem(Some(popup), 7 * 4).unwrap(),
            BM_CLICK,
            None,
            None,
        );
        let edit = take_property_edit(bar.0, 7).unwrap();
        assert_eq!(edit.context, 11);
        assert_eq!(edit.value.as_deref(), Some("2:2"));
        assert_eq!(edit.active, Some(false));
        // 一个消息结算前的失焦提交和两次开关点击必须保留最后的值与状态。
        SetWindowTextW(width, w!("5")).unwrap();
        SendMessageW(width, WM_KEYDOWN, Some(WPARAM(VK_RETURN.0 as usize)), None);
        for _ in 0..2 {
            SendMessageW(
                GetDlgItem(Some(popup), 7 * 4).unwrap(),
                BM_CLICK,
                None,
                None,
            );
        }
        let edit = take_property_edit(bar.0, 7).unwrap();
        assert_eq!(edit.value.as_deref(), Some("5:2"));
        assert_eq!(edit.active, Some(true));
        drop(bar);
        assert!(!IsWindow(Some(width)).as_bool());
        assert!(!IsWindow(Some(height)).as_bool());
    }
}
#[test]
fn ratio_rejects_invalid_schema_and_places_nonoverlapping_original_size_slots() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        for invalid in [
            "0:1", "1:0", "1:2:3", "NaN:1", "inf:1", "10001:1", "1e2:1", "+1:1",
        ] {
            assert!(set_properties(bar.0, 1, fields(invalid, false), true).is_err());
        }
        set_properties(bar.0, 1, fields("16:9", true), false).unwrap();
        let ui = toolbar::ui(bar.0).unwrap();
        let props = ui.properties.borrow();
        let data = &props.as_ref().unwrap().data;
        let controls = data.controls.borrow();
        let c = &controls[0];
        let px = |v| crate::layout::px(v, data.dpi.get());
        assert_eq!(c.frame.right - c.frame.left, px(103));
        let mut rects = Vec::new();
        for h in c.handles() {
            assert!(!IsWindowEnabled(h).as_bool());
            let mut rect = RECT::default();
            GetWindowRect(h, &mut rect).unwrap();
            rects.push(rect);
        }
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                assert!(
                    a.right <= b.left
                        || b.right <= a.left
                        || a.bottom <= b.top
                        || b.bottom <= a.top
                );
            }
        }
    }
}
