//! 系统 BUTTON 的实际计时/捕获与通知回归，消息仅发送给自有测试窗口。
use crate::tests::*;
use windows::Win32::{
    Foundation::*,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
unsafe fn fixture() -> (Window, Window, HWND) {
    let (owner, bar) = crate::tests::fixture();
    crate::prepare(bar.0).unwrap();
    let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
    let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
    let body = GetDlgItem(Some(bar.0), 1).unwrap();
    (owner, bar, body)
}
unsafe fn notification(owner: HWND) -> Option<(usize, isize)> {
    let mut m = MSG::default();
    PeekMessageW(&mut m, Some(owner), WM_APP + 1, WM_APP + 1, PM_REMOVE)
        .as_bool()
        .then_some((m.wParam.0, m.lParam.0))
}
#[test]
fn long_press_waits_450ms_releases_capture_and_suppresses_normal_click() {
    unsafe {
        let (owner, _bar, body) = fixture();
        SendMessageW(
            body,
            WM_LBUTTONDOWN,
            Some(WPARAM(1)),
            Some(LPARAM(5 | (5 << 16))),
        );
        assert_eq!(GetCapture(), body);
        assert!(notification(owner.0).is_none());
        let start = std::time::Instant::now();
        let mut timer = MSG::default();
        assert!(!PeekMessageW(&mut timer, Some(body), WM_TIMER, WM_TIMER, PM_REMOVE).as_bool());
        std::thread::sleep(std::time::Duration::from_millis(470));
        assert!(PeekMessageW(&mut timer, Some(body), WM_TIMER, WM_TIMER, PM_REMOVE).as_bool());
        assert!(start.elapsed().as_millis() >= 450);
        DispatchMessageW(&timer);
        assert!(GetCapture().is_invalid());
        let mut cmd = MSG::default();
        let parent = GetParent(body).unwrap();
        assert!(PeekMessageW(&mut cmd, Some(parent), WM_COMMAND, WM_COMMAND, PM_REMOVE).as_bool());
        DispatchMessageW(&cmd);
        assert_eq!(notification(owner.0), Some((1, 1)));
        SendMessageW(body, WM_LBUTTONUP, None, Some(LPARAM(5 | (5 << 16))));
        assert!(notification(owner.0).is_none());
    }
}
#[test]
fn movement_or_capture_loss_cancels_pending_long_press() {
    unsafe {
        let (owner, _bar, body) = fixture();
        for cancel in [WM_MOUSEMOVE, WM_CANCELMODE] {
            SendMessageW(
                body,
                WM_LBUTTONDOWN,
                Some(WPARAM(1)),
                Some(LPARAM(5 | (5 << 16))),
            );
            SendMessageW(
                body,
                cancel,
                Some(WPARAM(1)),
                Some(LPARAM(100 | (100 << 16))),
            );
            SendMessageW(body, WM_TIMER, Some(WPARAM(0x4e50)), None);
            let mut cmd = MSG::default();
            assert!(!PeekMessageW(
                &mut cmd,
                Some(GetParent(body).unwrap()),
                WM_COMMAND,
                WM_COMMAND,
                PM_REMOVE
            )
            .as_bool());
            let _ = ReleaseCapture();
            assert!(notification(owner.0).is_none());
        }
    }
}
