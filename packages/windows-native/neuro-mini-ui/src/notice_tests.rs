//! 只创建隐藏自有宿主；通知不抢焦点，销毁/超时后句柄凭证同步失效。
use super::*;
#[test]
fn child_notice_is_bounded_click_dismissible_and_cannot_destroy_a_reused_handle() {
    unsafe {
        let parent = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("notice fixture"),
            WS_POPUP,
            0,
            0,
            400,
            240,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let stack = NoticeStack::new(parent).unwrap();
        let notice = stack.push("已完成", "示例结果", "知道了").unwrap();
        let hwnd = notice.window().unwrap();
        assert_eq!(GetParent(hwnd).unwrap(), stack.window().unwrap());
        assert_eq!(
            GetWindow(stack.window().unwrap(), GW_OWNER).unwrap(),
            parent
        );
        SendMessageW(hwnd, WM_LBUTTONDOWN, None, None);
        SendMessageW(hwnd, WM_LBUTTONUP, None, None);
        assert!(notice.window().is_none());
        drop(notice);
        let notice = stack.push("过期", "不会阻塞输入", "关闭").unwrap();
        let hwnd = notice.window().unwrap();
        SendMessageW(hwnd, WM_TIMER, Some(WPARAM(1)), None);
        assert!(notice.window().is_none());
        let notice = stack.push("宿主关闭", "同步回收", "关闭").unwrap();
        DestroyWindow(parent).unwrap();
        assert!(notice.window().is_none());
        assert!(stack.window().is_none());
    }
}

#[path = "notice_stack_tests.rs"]
mod stacking;
