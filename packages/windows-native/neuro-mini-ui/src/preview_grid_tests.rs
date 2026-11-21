//! 自有 HWND 的卡片、局部滚动、凭证和父先销毁；不向桌面注入输入。
use super::*;
unsafe fn parent() -> HWND {
    CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        w!("STATIC"),
        w!("preview test"),
        WS_POPUP,
        0,
        0,
        400,
        400,
        None,
        None,
        None,
        None,
    )
    .unwrap()
}
fn cards(count: usize) -> Vec<PreviewCard> {
    (0..count)
        .map(|i| PreviewCard {
            title: format!("候选 {i}"),
            detail: format!("#{i}"),
            enabled: true,
            preview: None,
        })
        .collect()
}
#[test]
fn layout_is_fixed_density_bounded_and_independent_of_selection_at_each_dpi() {
    for dpi in [96, 120, 144, 192] {
        let l = PreviewGridLayout::new(64, dpi, f64::NAN);
        assert_eq!(l.width, px(250, dpi));
        assert_eq!(l.height, px(320, dpi));
        assert_eq!(l.cards[0].height, px(100, dpi));
        assert!(l.track().is_some() && l.thumb().unwrap().height >= px(18, dpi));
        let end = PreviewGridLayout::new(64, dpi, f64::MAX);
        assert_eq!(end.offset, (end.content - end.height) as f64);
        let short = PreviewGridLayout::new(2, dpi, 42.0);
        assert_eq!(short.offset, 0.0);
        assert!(short.track().is_none());
    }
}
#[test]
fn actual_buttons_keep_identity_and_emit_current_context_only_while_visible_and_enabled() {
    unsafe {
        let parent = parent();
        let grid = PreviewGrid::new(parent, WM_APP + 1, cards(2)).unwrap();
        let layout = grid
            .update("候选", "当前 #1", &[true, true], Some(0), 7, 96)
            .unwrap();
        let _ = ShowWindow(parent, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(grid.handle(), SW_SHOWNOACTIVATE);
        let button = GetDlgItem(Some(grid.handle()), 2).unwrap();
        SendMessageW(button, BM_CLICK, None, None);
        let mut msg = MSG::default();
        assert!(PeekMessageW(&mut msg, Some(parent), WM_APP + 1, WM_APP + 1, PM_REMOVE).as_bool());
        assert_eq!((msg.wParam.0, msg.lParam.0), (1, 7));
        assert_eq!(
            grid.update("候选", "当前 #2", &[true, false], Some(1), 8, 96)
                .unwrap(),
            layout
        );
        assert_eq!(GetDlgItem(Some(grid.handle()), 2).unwrap(), button);
        SendMessageW(button, BM_CLICK, None, None);
        assert!(!PeekMessageW(&mut msg, Some(parent), WM_APP + 1, WM_APP + 1, PM_REMOVE).as_bool());
        DestroyWindow(parent).unwrap();
        assert!(!grid.alive());
        assert!(grid.update("候选", "", &[true, true], None, 9, 96).is_err());
        drop(grid);
    }
}
#[test]
fn wheel_and_keyboard_reveal_scroll_only_the_same_grid_without_rebuilding_buttons() {
    unsafe {
        let parent = parent();
        let grid = PreviewGrid::new(parent, WM_APP + 1, cards(20)).unwrap();
        grid.update("候选", "", &[true; 20], None, 1, 96).unwrap();
        let button = GetDlgItem(Some(grid.handle()), 20).unwrap();
        SendMessageW(
            grid.handle(),
            WM_MOUSEWHEEL,
            Some(WPARAM(((-30i16) as u16 as usize) << 16)),
            None,
        );
        assert_eq!(grid.data().unwrap().layout.borrow().offset, 12.0);
        window::reveal(grid.handle(), grid.data().unwrap(), button);
        assert!(grid.data().unwrap().layout.borrow().offset > 12.0);
        assert_eq!(GetDlgItem(Some(grid.handle()), 20).unwrap(), button);
        drop(grid);
        DestroyWindow(parent).unwrap();
    }
}
#[test]
fn invalid_capacity_labels_and_context_are_rejected_at_public_boundary() {
    unsafe {
        let parent = parent();
        assert!(PreviewGrid::new(parent, WM_APP + 1, cards(65)).is_err());
        let mut invalid = cards(1);
        invalid[0].title = "bad\0title".into();
        assert!(PreviewGrid::new(parent, WM_APP + 1, invalid).is_err());
        let grid = PreviewGrid::new(parent, WM_APP + 1, cards(2)).unwrap();
        assert!(grid.update("候选", "", &[true, true], None, 0, 96).is_err());
        assert!(grid.update("候选", "", &[true], None, 1, 96).is_err());
        drop(grid);
        DestroyWindow(parent).unwrap();
    }
}

struct Buffer(HDC, HBITMAP, HGDIOBJ);
impl Buffer {
    unsafe fn new(width: i32, height: i32) -> Self {
        let dc = CreateCompatibleDC(None);
        assert!(!dc.is_invalid());
        let mut bits = std::ptr::null_mut();
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let bitmap = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0).unwrap();
        let old = SelectObject(dc, HGDIOBJ(bitmap.0));
        Self(dc, bitmap, old)
    }
}
impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.0, self.2);
            let _ = DeleteObject(HGDIOBJ(self.1 .0));
            let _ = DeleteDC(self.0);
        }
    }
}
struct PreparedPreview;
impl CardPreview for PreparedPreview {
    unsafe fn paint(&self, dc: HDC, rect: RECT) {
        SetDCBrushColor(dc, COLORREF(0x0044cc));
        FillRect(dc, &rect, HBRUSH(GetStockObject(DC_BRUSH).0));
    }
}
#[test]
fn prepared_preview_and_only_selected_card_border_paint_without_leaking_dc_or_font() {
    unsafe {
        let parent = parent();
        let mut cards = cards(2);
        cards[0].preview = Some(Rc::new(PreparedPreview));
        let grid = PreviewGrid::new(parent, WM_APP + 1, cards).unwrap();
        for dpi in [96, 120, 144, 192] {
            let layout = grid
                .update("候选", "当前 #1", &[true, true], Some(0), 1, dpi)
                .unwrap();
            let buffer = Buffer::new(layout.width, layout.height);
            paint::shell(buffer.0, grid.data().unwrap());
            GdiFlush().ok().unwrap();
            assert_eq!(GetPixel(buffer.0, layout.width / 2, 3), COLORREF(0x120d0a));
            for (index, frame) in layout.cards.iter().enumerate() {
                SetDCBrushColor(buffer.0, COLORREF(0x123456));
                let item = DRAWITEMSTRUCT {
                    hwndItem: GetDlgItem(Some(grid.handle()), index as i32 + 1).unwrap(),
                    hDC: buffer.0,
                    rcItem: RECT {
                        right: frame.width,
                        bottom: frame.height,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                paint::card(grid.data().unwrap(), &item);
                GdiFlush().ok().unwrap();
                assert_eq!(
                    GetPixel(buffer.0, 0, 0),
                    COLORREF(if index == 0 { 0x38ffd9 } else { 0x443a33 })
                );
                assert_eq!(
                    GetPixel(buffer.0, px(10, dpi), px(10, dpi)),
                    COLORREF(if index == 0 { 0x0044cc } else { 0x110c09 })
                );
                assert_eq!(GetDCBrushColor(buffer.0), COLORREF(0x123456));
            }
        }
        let font = grid.data().unwrap().font.get();
        DestroyWindow(parent).unwrap();
        assert!(!grid.alive());
        let mut info = LOGFONTW::default();
        assert_eq!(
            GetObjectW(
                HGDIOBJ(font.0),
                std::mem::size_of::<LOGFONTW>() as i32,
                Some((&mut info as *mut LOGFONTW).cast())
            ),
            0
        );
        drop(grid);
    }
}
