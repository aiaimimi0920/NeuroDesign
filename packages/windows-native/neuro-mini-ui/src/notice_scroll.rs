//! 通知栈和长通知共用的即时像素滚动；只持本次轨道捕获，不创建动画或后台任务。
use super::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetCapture, ReleaseCapture, SetCapture};
#[derive(Default)]
pub(super) struct Scroll {
    pub top: Cell<f64>,
    width: Cell<i32>,
    height: Cell<i32>,
    content: Cell<i32>,
    drag: Cell<Option<(i32, f64)>>,
}
impl Scroll {
    pub fn configure(&self, width: i32, height: i32, content: i32) {
        self.width.set(width);
        self.height.set(height);
        self.content.set(content);
        self.set(self.top.get());
    }
    pub fn maximum(&self) -> f64 {
        (self.content.get() - self.height.get()).max(0) as f64
    }
    pub fn set(&self, top: f64) -> bool {
        self.top.replace(top.clamp(0.0, self.maximum())) != self.top.get()
    }
    pub fn track(&self, dpi: u32) -> RECT {
        RECT {
            left: (self.width.get() - crate::layout::px(8, dpi)).max(0),
            top: 0,
            right: self.width.get(),
            bottom: self.height.get(),
        }
    }
    fn thumb(&self, dpi: u32) -> RECT {
        let mut r = self.track(dpi);
        let size = ((r.bottom as f64 * r.bottom as f64 / self.content.get().max(1) as f64).round()
            as i32)
            .max(crate::layout::px(18, dpi))
            .min(r.bottom);
        r.top =
            ((r.bottom - size) as f64 * self.top.get() / self.maximum().max(1.0)).round() as i32;
        r.bottom = r.top + size;
        r
    }
    pub unsafe fn cancel(&self, hwnd: HWND) {
        if self.drag.replace(None).is_some() && GetCapture() == hwnd {
            let _ = ReleaseCapture();
        }
    }
    pub unsafe fn input(&self, hwnd: HWND, dpi: u32, msg: u32, wp: WPARAM, lp: LPARAM) -> bool {
        let point = POINT {
            x: lp.0 as i16 as i32,
            y: (lp.0 >> 16) as i16 as i32,
        };
        match msg {
            WM_MOUSEWHEEL => {
                return self.set(
                    self.top.get()
                        - (wp.0 >> 16) as i16 as f64 * crate::layout::px(30, dpi) as f64 / 120.0,
                );
            }
            WM_LBUTTONDOWN if self.maximum() > 0.0 => {
                if !PtInRect(&self.track(dpi), point).as_bool() {
                    return false;
                }
                let thumb = self.thumb(dpi);
                if point.y < thumb.top || point.y >= thumb.bottom {
                    self.set(
                        self.top.get()
                            + if point.y < thumb.top { -1.0 } else { 1.0 }
                                * self.height.get() as f64,
                    );
                }
                self.drag.set(Some((point.y, self.top.get())));
                let _ = SetCapture(hwnd);
            }
            WM_MOUSEMOVE if self.drag.get().is_some() => {
                let (y, top) = self.drag.get().unwrap();
                let thumb = self.thumb(dpi);
                let travel = self.height.get() - (thumb.bottom - thumb.top);
                if travel > 0 {
                    self.set(top + (point.y - y) as f64 * self.maximum() / travel as f64);
                }
            }
            WM_LBUTTONUP if self.drag.get().is_some() => self.cancel(hwnd),
            WM_CANCELMODE | WM_CAPTURECHANGED => {
                self.cancel(hwnd);
                return false;
            }
            WM_KEYDOWN if self.maximum() > 0.0 => {
                let top = self.top.get();
                let row = crate::layout::px(30, dpi) as f64;
                let page = self.height.get() as f64;
                let next = match wp.0 {
                    0x26 => top - row,
                    0x28 => top + row,
                    0x21 => top - page,
                    0x22 => top + page,
                    0x24 => 0.0,
                    0x23 => self.maximum(),
                    _ => return false,
                };
                self.set(next);
            }
            _ => return false,
        }
        true
    }
    pub unsafe fn paint(&self, dc: HDC, dpi: u32) {
        if self.maximum() == 0.0 {
            return;
        }
        let p = crate::Palette::default();
        let brush = CreateSolidBrush(paint::color(crate::paint::mix(p.control, p.text, 0.24)));
        FillRect(dc, &self.thumb(dpi), brush);
        let _ = DeleteObject(HGDIOBJ(brush.0));
    }
}
