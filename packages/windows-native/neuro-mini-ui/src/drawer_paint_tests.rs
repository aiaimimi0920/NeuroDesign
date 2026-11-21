//! 自有内存 DIB 验证真实按钮绘制与局部激活色，不读取桌面像素。
use super::*;
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
#[test]
fn shell_caret_and_buttons_paint_at_each_dpi_and_only_the_target_is_active() {
    unsafe {
        let owner = tests::parent();
        let mut buttons = tests::buttons();
        let drawer = Drawer::new(owner, WM_APP + 1, &buttons).unwrap();
        for dpi in [96, 120, 144, 192] {
            let layout = drawer.update("属性", &buttons, true, 1, dpi).unwrap();
            let buffer = Buffer::new(layout.width, layout.height);
            let data = drawer.data().unwrap();
            data.painter
                .borrow()
                .shell(buffer.0, &layout, dpi, true)
                .unwrap();
            GdiFlush().ok().unwrap();
            assert_eq!(GetPixel(buffer.0, layout.width / 2, 3), COLORREF(0x120d0a));
            for (i, button) in buttons.iter_mut().enumerate() {
                let hwnd = GetDlgItem(Some(drawer.handle()), button.id as i32).unwrap();
                let frame = layout.buttons[i];
                let item = DRAWITEMSTRUCT {
                    hwndItem: hwnd,
                    hDC: buffer.0,
                    rcItem: RECT {
                        left: 0,
                        top: 0,
                        right: frame.width,
                        bottom: frame.height,
                    },
                    ..Default::default()
                };
                for active in [false, true, false] {
                    button.active = active;
                    data.painter
                        .borrow()
                        .button(&item, Some(button), dpi, true)
                        .unwrap();
                    GdiFlush().ok().unwrap();
                    let rgb = if active {
                        crate::paint::mix(0x0e1218, 0xd9ff38, 0.14)
                    } else {
                        0x20262d
                    };
                    let expected =
                        COLORREF(((rgb & 0xff) << 16) | (rgb & 0xff00) | ((rgb >> 16) & 0xff));
                    assert_eq!(
                        GetPixel(buffer.0, frame.width / 2, crate::layout::px(3, dpi)),
                        expected
                    );
                }
            }
            let caret = DRAWITEMSTRUCT {
                hwndItem: data.caret,
                hDC: buffer.0,
                rcItem: RECT {
                    left: 0,
                    top: 0,
                    right: layout.caret.width,
                    bottom: layout.caret.height,
                },
                ..Default::default()
            };
            data.painter
                .borrow()
                .button(&caret, None, dpi, true)
                .unwrap();
            GdiFlush().ok().unwrap();
            assert_eq!(
                GetPixel(buffer.0, layout.caret.width / 2, layout.caret.height / 2),
                COLORREF(0x38ffd9)
            );
        }
        drop(drawer);
        DestroyWindow(owner).unwrap();
    }
}
