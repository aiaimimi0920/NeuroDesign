//! 真实子窗口证明局部滚动、回收重排、捕获排除和同步跟随；不采集桌面或发送全局输入。
use super::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetCapture, GetFocus};
struct Owner(HWND);
impl Drop for Owner {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}
unsafe fn owner(height: i32) -> Owner {
    Owner(
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("notice stack fixture"),
            WS_POPUP,
            100,
            200,
            400,
            height,
            None,
            None,
            None,
            None,
        )
        .unwrap(),
    )
}
unsafe fn rect(hwnd: HWND) -> RECT {
    let mut r = RECT::default();
    GetWindowRect(hwnd, &mut r).unwrap();
    r
}
unsafe fn wheel(hwnd: HWND, delta: i16) {
    SendMessageW(
        hwnd,
        WM_MOUSEWHEEL,
        Some(WPARAM((delta as u16 as usize) << 16)),
        None,
    );
}
unsafe fn ui(notice: &Notice) -> &Ui {
    &*(GetWindowLongPtrW(notice.window().unwrap(), GWLP_USERDATA) as *const Ui)
}
#[test]
fn bounded_viewport_scrolls_stable_cards_and_leaves_gaps_click_through() {
    unsafe {
        let owner = owner(220);
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let stack = NoticeStack::new(owner.0).unwrap();
        let notices: Vec<_> = (0..4)
            .map(|n| {
                stack
                    .push(&format!("通知 {n}"), &"结果内容 ".repeat(100), "知道了")
                    .unwrap()
            })
            .collect();
        let root = stack.window().unwrap();
        let dpi = GetDpiForWindow(owner.0).max(96);
        let px = |n| crate::layout::px(n, dpi);
        let r = rect(root);
        assert!(r.bottom <= rect(owner.0).bottom - px(8));
        assert!(r.right <= rect(owner.0).right - px(8));
        assert_eq!(r.right - r.left, px(280));
        let first = notices[3].window().unwrap();
        let next = notices[2].window().unwrap();
        let initial = rect(first);
        assert_eq!(initial.top, r.top);
        assert_eq!(rect(next).top, initial.bottom + px(8));
        let region = CreateRectRgn(0, 0, 0, 0);
        assert_ne!(GetWindowRgn(root, region), RGN_ERROR);
        assert!(!PtInRegion(region, px(5), initial.bottom - r.top + px(3)).as_bool());
        let _ = DeleteObject(HGDIOBJ(region.0));
        wheel(root, -60);
        assert_eq!(
            rect(first).top,
            initial.top - (px(30) as f64 / 2.0).round() as i32
        );
        assert_eq!(notices[3].window(), Some(first));
        SendMessageW(root, WM_KEYDOWN, Some(WPARAM(0x23)), None);
        assert_eq!(rect(notices[0].window().unwrap()).bottom, r.bottom);
        SendMessageW(root, WM_KEYDOWN, Some(WPARAM(0x24)), None);
        assert_eq!(rect(first).top, r.top);
        SendMessageW(first, WM_KEYDOWN, Some(WPARAM(27)), None);
        assert!(notices[3].window().is_none());
        assert_eq!(
            rect(next).top,
            r.top,
            "close reflows synchronously without a host tick"
        );
        let _extra = stack.push("最新", "新通知重置到顶部", "关闭").unwrap();
        assert_eq!(rect(_extra.window().unwrap()).top, rect(root).top);
    }
}
#[test]
fn long_card_scrolls_internally_then_chains_to_stack_without_dismiss_or_pin_input() {
    unsafe {
        let owner = owner(200);
        let stack = NoticeStack::new(owner.0).unwrap();
        let old = stack.push("旧通知", "示例", "关闭").unwrap();
        let notice = stack
            .push("长通知", &"需要完整阅读的通知文本 ".repeat(100), "知道了")
            .unwrap();
        let root = stack.window().unwrap();
        let card = notice.window().unwrap();
        let before = rect(card);
        wheel(card, -60);
        assert!(ui(&notice).scroll.top.get() > 0.0);
        assert_eq!(
            rect(card),
            before,
            "card wheel must not scroll the outer stack first"
        );
        SendMessageW(card, WM_KEYDOWN, Some(WPARAM(0x23)), None);
        assert_eq!(ui(&notice).scroll.top.get(), ui(&notice).scroll.maximum());
        wheel(card, -120);
        assert!(
            rect(card).top < before.top,
            "at the inner edge, wheel continues the outer stack"
        );
        let track = ui(&notice).scroll.track(GetDpiForWindow(owner.0).max(96));
        let point = LPARAM((track.left as u16 as isize) | ((10u16 as isize) << 16));
        SendMessageW(card, WM_LBUTTONDOWN, None, Some(point));
        assert_eq!(GetCapture(), card);
        SendMessageW(card, WM_LBUTTONUP, None, Some(point));
        assert!(notice.window().is_some() && GetCapture().is_invalid());
        SendMessageW(root, WM_KEYDOWN, Some(WPARAM(0x23)), None);
        drop(old);
        assert_eq!(rect(card).top, rect(root).top);
        assert!(ui(&notice).scroll.maximum() > 0.0);
    }
}
#[test]
fn owner_move_hide_resize_affinity_and_timer_cleanup_are_immediate_without_activation() {
    unsafe {
        let owner = owner(260);
        let stack = NoticeStack::new(owner.0).unwrap();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let foreground = GetForegroundWindow();
        let focus = GetFocus();
        let one = stack.push("完成", "结果一", "知道了").unwrap();
        let two = stack.push("完成", "结果二", "知道了").unwrap();
        let root = stack.window().unwrap();
        let before = rect(root);
        SetWindowPos(
            owner.0,
            None,
            250,
            350,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
        .unwrap();
        let after = rect(root);
        assert_eq!(
            (after.left - before.left, after.top - before.top),
            (150, 150)
        );
        assert_eq!(GetForegroundWindow(), foreground);
        assert_eq!(GetFocus(), focus);
        SendMessageW(two.window().unwrap(), WM_LBUTTONDOWN, None, None);
        assert_eq!(GetCapture(), two.window().unwrap());
        let _ = ShowWindow(owner.0, SW_HIDE);
        assert!(!IsWindowVisible(root).as_bool());
        assert!(
            GetCapture().is_invalid(),
            "hiding owner releases even a card content press"
        );
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        assert!(IsWindowVisible(root).as_bool());
        SetWindowDisplayAffinity(owner.0, WDA_EXCLUDEFROMCAPTURE).unwrap();
        stack.place().unwrap();
        let mut affinity = 0;
        GetWindowDisplayAffinity(root, &mut affinity).unwrap();
        assert_eq!(affinity, WDA_EXCLUDEFROMCAPTURE.0);
        SetWindowPos(
            owner.0,
            None,
            0,
            0,
            150,
            100,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
        .unwrap();
        assert!(rect(root).right <= rect(owner.0).right);
        assert!(rect(root).bottom <= rect(owner.0).bottom);
        let second = two.window().unwrap();
        SendMessageW(second, WM_TIMER, Some(WPARAM(1)), None);
        assert!(two.window().is_none());
        assert_eq!(rect(one.window().unwrap()).top, rect(root).top);
        drop(one);
        assert!(!IsWindowVisible(root).as_bool());
    }
}
#[test]
fn invalid_content_does_not_replace_live_cards_and_card_click_needs_a_real_press() {
    unsafe {
        let owner = owner(240);
        let stack = NoticeStack::new(owner.0).unwrap();
        let notice = stack.push("有效", "内容", "关闭").unwrap();
        for (title, text) in [
            ("\0".to_owned(), "正文".to_owned()),
            ("😀".repeat(65), "正文".to_owned()),
            ("标题".to_owned(), "a".repeat(2049)),
        ] {
            assert!(stack.push(&title, &text, "关闭").is_err());
        }
        let hwnd = notice.window().unwrap();
        SendMessageW(hwnd, WM_LBUTTONUP, None, None);
        assert_eq!(notice.window(), Some(hwnd));
        SendMessageW(hwnd, WM_TIMER, Some(WPARAM(99)), None);
        assert_eq!(notice.window(), Some(hwnd));
        SendMessageW(hwnd, WM_LBUTTONDOWN, None, None);
        SendMessageW(hwnd, WM_LBUTTONUP, None, None);
        assert!(notice.window().is_none());
        assert!(stack.window().is_some());
    }
}

unsafe extern "system" fn count_wheel(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    data: usize,
) -> LRESULT {
    if msg == WM_MOUSEWHEEL {
        (*(data as *const Cell<u32>)).set((*(data as *const Cell<u32>)).get() + 1);
    }
    windows::Win32::UI::Shell::DefSubclassProc(hwnd, msg, wp, lp)
}
#[test]
fn exhausted_wheel_never_reaches_pin_and_unchanged_sync_does_not_schedule_more_paint() {
    unsafe {
        let count = Cell::new(0u32);
        let owner = owner(220);
        assert!(windows::Win32::UI::Shell::SetWindowSubclass(
            owner.0,
            Some(count_wheel),
            99,
            &count as *const Cell<u32> as usize
        )
        .as_bool());
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let stack = NoticeStack::new(owner.0).unwrap();
        let notice = stack.push("短通知", "不改变钉图", "关闭").unwrap();
        let root = stack.window().unwrap();
        let card = notice.window().unwrap();
        for delta in [-120, 120] {
            wheel(card, delta);
            wheel(root, delta);
        }
        assert_eq!(count.get(), 0);
        assert!(ValidateRect(Some(root), None).as_bool());
        assert!(ValidateRect(Some(card), None).as_bool());
        for _ in 0..5 {
            stack.place().unwrap();
        }
        assert!(!GetUpdateRect(root, None, false).as_bool());
        assert!(!GetUpdateRect(card, None, false).as_bool());
        assert!(
            windows::Win32::UI::Shell::RemoveWindowSubclass(owner.0, Some(count_wheel), 99)
                .as_bool()
        );
    }
}
