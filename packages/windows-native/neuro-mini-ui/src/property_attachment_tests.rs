//! 零缝拼接、上下翻转与原子批次接线；真实 EDIT/owner 生命周期仍沿原属性测试。
use super::*;
use windows::Win32::UI::Shell::*;
fn rect(x: i32, y: i32, w: i32, h: i32) -> RECT {
    RECT {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}
#[test]
fn attached_rows_have_no_gap_above_or_below_on_negative_monitors_and_each_dpi() {
    for dpi in [96, 120, 144, 192] {
        let h = crate::layout::px(38, dpi);
        let work = rect(-1920, -300, 1920, 1040);
        let above = rect(-1700, 100, 300, h);
        assert_eq!(planned(above, work, 400, h, dpi).bottom, above.top);
        let below = rect(-1700, -300, 300, h);
        assert_eq!(planned(below, work, 400, h, dpi).top, below.bottom);
    }
}
unsafe extern "system" fn observe(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    data: usize,
) -> LRESULT {
    if msg == WM_WINDOWPOSCHANGED && lp.0 != 0 {
        (*(data as *mut Vec<SET_WINDOW_POS_FLAGS>)).push((*(lp.0 as *const WINDOWPOS)).flags);
    }
    DefSubclassProc(hwnd, msg, wp, lp)
}
#[test]
fn host_batch_moves_both_rows_without_independent_property_follow_and_restores_round_shell() {
    unsafe {
        let (owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        SetWindowPos(
            bar.0,
            None,
            300,
            400,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
        .unwrap();
        set_properties(
            bar.0,
            1,
            vec![PropertyField {
                id: 1,
                label: "粗细",
                icon: None,
                kind: PropertyKind::Number {
                    value: 3,
                    min: 0,
                    max: 99,
                    suffix: "",
                },
            }],
            true,
        )
        .unwrap();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
        let mut main = RECT::default();
        GetWindowRect(bar.0, &mut main).unwrap();
        let next = rect(
            main.left + 80,
            main.top + 50,
            main.right - main.left,
            main.bottom - main.top,
        );
        let (property, expected) = attached_position(bar.0, next).unwrap().unwrap();
        let mut changes: Vec<SET_WINDOW_POS_FLAGS> = Vec::new();
        SetWindowSubclass(
            property,
            Some(observe),
            602,
            &mut changes as *mut Vec<SET_WINDOW_POS_FLAGS> as usize,
        )
        .ok()
        .unwrap();
        let flags = SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOREDRAW;
        let mut batch = BeginDeferWindowPos(2).unwrap();
        for (hwnd, r) in [(bar.0, next), (property, expected)] {
            batch = DeferWindowPos(batch, hwnd, None, r.left, r.top, 0, 0, flags).unwrap();
        }
        EndDeferWindowPos(batch).unwrap();
        sync_attached(bar.0).unwrap();
        RemoveWindowSubclass(property, Some(observe), 602)
            .ok()
            .unwrap();
        assert_eq!(
            changes.len(),
            1,
            "property must not follow in a separate position call"
        );
        assert!(changes[0].contains(SWP_NOREDRAW));
        let mut actual = RECT::default();
        GetWindowRect(property, &mut actual).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.bottom, next.top);
        let r = CreateRectRgn(0, 0, 0, 0);
        GetWindowRgn(bar.0, r);
        assert!(PtInRegion(r, 0, 0).as_bool());
        set_properties(bar.0, 2, Vec::new(), true).unwrap();
        GetWindowRgn(bar.0, r);
        assert!(!PtInRegion(r, 0, 0).as_bool());
        DeleteObject(HGDIOBJ(r.0)).ok().unwrap();
    }
}
