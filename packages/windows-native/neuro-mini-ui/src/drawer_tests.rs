//! 自有隐藏宿主验证真实控件、context、原位更新和回收；不注入桌面输入。
use super::*;
pub(super) unsafe fn parent() -> HWND {
    CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        w!("STATIC"),
        w!("drawer test"),
        WS_POPUP,
        0,
        0,
        400,
        200,
        None,
        None,
        None,
        None,
    )
    .unwrap()
}
pub(super) fn buttons() -> Vec<DrawerButton> {
    vec![
        DrawerButton {
            id: 2,
            label: "开关".into(),
            icon: Icon::stroke("M2 2h20v20H2Z"),
            text: None,
            active: false,
            enabled: true,
        },
        DrawerButton {
            id: 3,
            label: "应用".into(),
            icon: Icon::stroke("M4 12h16"),
            text: Some("应用".into()),
            active: false,
            enabled: true,
        },
    ]
}
#[test]
fn drawer_geometry_has_baseline_density_and_centered_caret_at_each_dpi() {
    for dpi in [96, 120, 144, 192] {
        let open = DrawerLayout::new(100, &[false, false, true], true, dpi);
        assert_eq!(open.height, crate::layout::px(52, dpi));
        assert_eq!(open.buttons[0].height, crate::layout::px(28, dpi));
        assert_eq!(open.caret.x, (open.width - open.caret.width) / 2);
        let closed = DrawerLayout::new(100, &[false, false, true], false, dpi);
        assert_eq!(
            (closed.width, closed.height),
            (open.width, crate::layout::px(10, dpi))
        );
        assert_eq!(closed.caret.x, open.caret.x);
    }
}
#[test]
fn real_drawer_retains_button_identity_focus_and_context_without_a_new_top_level() {
    unsafe {
        let parent = parent();
        let drawer = Drawer::new(parent, WM_APP + 1, &buttons()).unwrap();
        let mut spec = buttons();
        drawer
            .update("很长的属性名称仍保留完整可访问名称", &spec, true, 7, 96)
            .unwrap();
        let root = drawer.handle();
        let button = GetDlgItem(Some(root), 2).unwrap();
        assert_eq!(GetParent(root).unwrap(), parent);
        assert!(GetWindowLongW(root, GWL_STYLE) as u32 & WS_CHILD.0 != 0);
        let _ = SetFocus(Some(button));
        let focus = GetFocus();
        spec[0].active = true;
        drawer
            .update("很长的属性名称仍保留完整可访问名称", &spec, true, 8, 96)
            .unwrap();
        assert_eq!(GetDlgItem(Some(root), 2).unwrap(), button);
        assert_eq!(GetFocus(), focus);
        SendMessageW(button, BM_CLICK, None, None);
        let mut msg = MSG::default();
        assert!(PeekMessageW(&mut msg, Some(parent), WM_APP + 1, WM_APP + 1, PM_REMOVE).as_bool());
        assert_eq!(
            (msg.wParam.0 & 0xffff, msg.wParam.0 >> 32, msg.lParam.0),
            (2, 8, root.0 as isize)
        );
        let data = drawer.data().unwrap();
        let context = data.context.get();
        SendMessageW(
            root,
            WM_COMMAND,
            Some(WPARAM(2)),
            Some(LPARAM(data.caret.0 as isize)),
        );
        assert!(!PeekMessageW(&mut msg, Some(parent), WM_APP + 1, WM_APP + 1, PM_REMOVE).as_bool());
        drawer.update("属性", &spec, false, context, 144).unwrap();
        assert!(GetWindowLongW(button, GWL_STYLE) as u32 & WS_VISIBLE.0 == 0);
        DestroyWindow(parent).unwrap();
        assert!(!IsWindow(Some(root)).as_bool());
        drop(drawer);
    }
}
#[test]
fn invalid_ids_and_changed_schema_fail_before_reinterpreting_controls() {
    unsafe {
        let parent = parent();
        let mut spec = buttons();
        spec[0].id = DRAWER_TOGGLE;
        assert!(Drawer::new(parent, WM_APP + 1, &spec).is_err());
        let drawer = Drawer::new(parent, WM_APP + 1, &buttons()).unwrap();
        assert!(drawer.update("属性", &buttons(), true, 0, 96).is_err());
        assert!(drawer.update("属性", &spec, true, 1, 96).is_err());
        drop(drawer);
        DestroyWindow(parent).unwrap();
    }
}

#[test]
fn collapsed_root_and_host_exclude_black_rectangle_and_expansion_restores_body() {
    unsafe {
        let parent = parent();
        let drawer = Drawer::new(parent, WM_APP + 1, &buttons()).unwrap();
        for dpi in [96, 120, 144, 192] {
            for expanded in [false, true, false] {
                let layout = drawer.update("属性", &buttons(), expanded, 1, dpi).unwrap();
                layout.apply_region(parent, expanded, dpi).unwrap();
                for hwnd in [parent, drawer.handle()] {
                    let region = CreateRectRgn(0, 0, 0, 0);
                    assert_ne!(GetWindowRgn(hwnd, region), RGN_ERROR);
                    assert_eq!(PtInRegion(region, 0, 0).as_bool(), expanded);
                    assert!(
                        PtInRegion(region, layout.width / 2, crate::layout::px(3, dpi)).as_bool()
                    );
                    DeleteObject(HGDIOBJ(region.0)).unwrap();
                }
            }
        }
        drop(drawer);
        DestroyWindow(parent).unwrap();
    }
}
