//! 使用真实自有 HWND 和离屏 DC 检查组件；不操作用户桌面或注入输入。
use super::*;
use windows::{
    core::w,
    Win32::{
        Foundation::*,
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
};
const ITEMS: &[Item] = &[
    Item::Button(Button {
        id: 1,
        label: "选择",
        icon: Icon::stroke("M5 3v17l5-6h9Z"),
        dropdown: true,
    }),
    Item::Divider,
    Item::Button(Button {
        id: 2,
        label: "撤销",
        icon: Icon::stroke("M8 5l-5 5 5 5"),
        dropdown: false,
    }),
];
pub(crate) struct Window(pub(crate) HWND);
impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.0).ok();
        }
    }
}
pub(crate) unsafe fn fixture() -> (Window, Window) {
    let owner = Window(
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("STATIC"),
            w!("theme fixture"),
            WS_POPUP,
            100,
            150,
            300,
            200,
            None,
            None,
            None,
            None,
        )
        .unwrap(),
    );
    let bar = Window(create(owner.0, "工具", WM_APP + 1, ITEMS).unwrap());
    (owner, bar)
}
#[test]
fn geometry_matches_main_strip_metrics_and_wraps_without_changing_slot_size() {
    for dpi in [96, 120, 144, 192, 288] {
        let layout = Layout::new(ITEMS, dpi, i32::MAX);
        assert_eq!(layout.width, layout::px(76, dpi));
        assert_eq!(layout.height, layout::px(38, dpi));
        assert_eq!(layout.slots[0].width, layout::px(28, dpi));
        let narrow = Layout::new(ITEMS, dpi, layout::px(40, dpi));
        assert!(narrow.slots[1].y > narrow.slots[0].y);
        assert_eq!(narrow.slots[1].width, layout.slots[1].width);
    }
}
#[test]
fn native_toolbar_has_no_caption_keeps_real_button_names_and_releases_children() {
    unsafe {
        let (owner, bar) = fixture();
        assert!(!ready(bar.0));
        assert!(GetDlgItem(Some(bar.0), 1).is_err());
        prepare(bar.0).unwrap();
        prepare(bar.0).unwrap();
        assert_eq!(super::owner(bar.0), Some(owner.0));
        assert_eq!(
            GetWindowLongW(bar.0, GWL_STYLE) as u32
                & (WS_CAPTION | WS_SYSMENU | WS_HSCROLL | WS_VSCROLL).0,
            0
        );
        let button = GetDlgItem(Some(bar.0), 1).unwrap();
        let corner = GetDlgItem(Some(bar.0), 0x8001).unwrap();
        update(
            bar.0,
            1,
            Icon::stroke("M5 6h14v12H5Z"),
            true,
            false,
            false,
            "矩形",
        )
        .unwrap();
        assert!(!IsWindowEnabled(button).as_bool() && !IsWindowEnabled(corner).as_bool());
        let mut text = [0; 32];
        let n = GetWindowTextW(button, &mut text) as usize;
        assert_eq!(String::from_utf16_lossy(&text[..n]), "矩形");
        let handle = bar.0;
        drop(bar);
        assert!(
            !IsWindow(Some(handle)).as_bool()
                && !IsWindow(Some(button)).as_bool()
                && !IsWindow(Some(corner)).as_bool()
        );
    }
}
#[test]
fn body_and_corner_post_distinct_notifications_without_executing_application_code() {
    unsafe {
        let (owner, bar) = fixture();
        prepare(bar.0).unwrap();
        for (id, dropdown) in [(1, 0), (0x8001, 1)] {
            SendMessageW(GetDlgItem(Some(bar.0), id).unwrap(), BM_CLICK, None, None);
            let mut message = MSG::default();
            assert!(PeekMessageW(
                &mut message,
                Some(owner.0),
                WM_APP + 1,
                WM_APP + 1,
                PM_REMOVE
            )
            .as_bool());
            assert_eq!((message.wParam.0, message.lParam.0), (1, dropdown));
            assert!(!PeekMessageW(
                &mut message,
                Some(owner.0),
                WM_APP + 1,
                WM_APP + 1,
                PM_REMOVE
            )
            .as_bool());
        }
    }
}
#[test]
fn prepared_toolbar_reflows_for_width_and_dpi_without_replacing_controls() {
    unsafe {
        let (_owner, bar) = fixture();
        prepare(bar.0).unwrap();
        let first = GetDlgItem(Some(bar.0), 1).unwrap();
        let second = GetDlgItem(Some(bar.0), 2).unwrap();
        for (dpi, width) in [(96, 40), (144, 1000), (120, 50)] {
            let layout = Layout::new(ITEMS, dpi, width);
            resize(bar.0, width, dpi).unwrap();
            resize(bar.0, width, dpi).unwrap();
            let mut rect = RECT::default();
            GetWindowRect(bar.0, &mut rect).unwrap();
            assert_eq!(rect.right - rect.left, layout.width);
            assert_eq!(rect.bottom - rect.top, layout.height);
            assert_eq!(first, GetDlgItem(Some(bar.0), 1).unwrap());
            assert_eq!(second, GetDlgItem(Some(bar.0), 2).unwrap());
            for (child, slot) in [first, second].into_iter().zip(&layout.slots) {
                let mut child_rect = RECT::default();
                GetWindowRect(child, &mut child_rect).unwrap();
                assert_eq!(child_rect.left - rect.left, slot.x);
                assert_eq!(child_rect.top - rect.top, slot.y);
            }
        }
    }
}
#[test]
fn path_renderer_accepts_real_icons_and_rejects_unsupported_segments() {
    unsafe {
        let factory: windows::Win32::Graphics::Direct2D::ID2D1Factory =
            windows::Win32::Graphics::Direct2D::D2D1CreateFactory(
                windows::Win32::Graphics::Direct2D::D2D1_FACTORY_TYPE_SINGLE_THREADED,
                None,
            )
            .unwrap();
        assert!(path::geometry(
            &factory,
            "M12 2C10 6 5 11 5 15a7 7 0 0 0 14 0c0-4-5-9-7-13Z",
            1.0,
            0.0,
            false
        )
        .is_ok());
        assert!(path::geometry(&factory, "M0 0Q2 2 4 0", 1.0, 0.0, false).is_err());
        assert!(path::geometry(&factory, "not a path", 1.0, 0.0, false).is_err());
    }
}
