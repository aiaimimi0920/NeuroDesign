//! 下附着变体的 28 DIP 按钮、20 DIP 原图标与 11 DIP 文字；不加载外部字体或图像。
use super::*;
use crate::{
    layout::px,
    paint::{color, mix},
};
use windows::Win32::Graphics::Direct2D::{Common::*, *};
pub(super) struct Paint {
    p: Painter,
    font: Option<(u32, HFONT)>,
    pub background: HBRUSH,
}
impl Paint {
    pub unsafe fn new() -> Result<Self> {
        let p = Painter::new()?;
        let background = CreateSolidBrush(COLORREF(0x120d0a));
        if background.is_invalid() {
            return Err(Error::from_win32());
        }
        Ok(Self {
            p,
            font: None,
            background,
        })
    }
    pub unsafe fn font(&mut self, dpi: u32) -> Result<HFONT> {
        if let Some((_, font)) = self.font.filter(|(old, _)| *old == dpi) {
            return Ok(font);
        }
        let font = CreateFontW(
            -px(11, dpi),
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe UI"),
        );
        if font.is_invalid() {
            return Err(Error::from_win32());
        }
        if let Some((_, old)) = self.font.replace((dpi, font)) {
            let _ = DeleteObject(HGDIOBJ(old.0));
        }
        Ok(font)
    }
    pub unsafe fn title_width(&mut self, hwnd: HWND, text: &str, dpi: u32) -> Result<i32> {
        let font = self.font(dpi)?;
        let dc = GetDC(Some(hwnd));
        if dc.is_invalid() {
            return Err(Error::from_win32());
        }
        let old = SelectObject(dc, HGDIOBJ(font.0));
        let mut size = SIZE::default();
        let result =
            GetTextExtentPoint32W(dc, &text.encode_utf16().collect::<Vec<_>>(), &mut size).ok();
        SelectObject(dc, old);
        ReleaseDC(Some(hwnd), dc);
        result?;
        Ok(size.cx.min(px(88, dpi)) + px(12, dpi))
    }
    pub unsafe fn shell(
        &self,
        dc: HDC,
        layout: &DrawerLayout,
        dpi: u32,
        expanded: bool,
    ) -> Result<()> {
        let p = &self.p;
        let scale = dpi.max(96) as f32 / 96.0;
        let brush = p
            .target
            .CreateSolidColorBrush(&color(0x333a44, 1.0), None)?;
        p.begin(
            dc,
            RECT {
                left: 0,
                top: 0,
                right: layout.width,
                bottom: layout.height,
            },
            0x0a0d12,
        )?;
        if expanded {
            p.target.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: 0.5 * scale,
                        top: 0.0,
                        right: layout.width as f32 - 0.5 * scale,
                        bottom: layout.height as f32 - 0.5 * scale,
                    },
                    radiusX: 4.0 * scale,
                    radiusY: 4.0 * scale,
                },
                &brush,
                scale,
                None,
            );
        }
        p.target.EndDraw(None, None)
    }
    pub unsafe fn button(
        &self,
        item: &DRAWITEMSTRUCT,
        spec: Option<&DrawerButton>,
        dpi: u32,
        expanded: bool,
    ) -> Result<()> {
        let p = &self.p;
        let scale = dpi.max(96) as f32 / 96.0;
        let width = item.rcItem.right - item.rcItem.left;
        let height = item.rcItem.bottom - item.rcItem.top;
        let active = spec.is_some_and(|s| s.active);
        let enabled = IsWindowEnabled(item.hwndItem).as_bool();
        let hot = crate::button::hot(item.hwndItem);
        let pressed = item.itemState.0 & ODS_SELECTED.0 != 0;
        let focused = item.itemState.0 & ODS_FOCUS.0 != 0;
        let ink = if active || spec.is_none() {
            0xd9ff38
        } else if hot {
            0xffffff
        } else {
            0xcbd5e1
        };
        let background = if active {
            mix(0x0e1218, 0xd9ff38, 0.14)
        } else if pressed {
            0x20262d
        } else if hot {
            0x303943
        } else {
            0x20262d
        };
        let icon = if let Some(spec) = spec {
            spec.icon
        } else {
            Icon::svg(if expanded {
                "<svg viewBox=\"0 0 14 14\" fill=\"currentColor\"><path d=\"M0 8h14L7 0Z\"/></svg>"
            } else {
                "<svg viewBox=\"0 0 14 14\" fill=\"currentColor\"><path d=\"M0 0h14L7 8Z\"/></svg>"
            })
        };
        // SVG 解析统一归一化到24；三角仍保持14×8，普通图标保持20×20。
        let size = if spec.is_some() { 20.0 } else { 14.0 } * scale;
        let icon = crate::icon::prepare(
            p,
            icon,
            size / 24.0,
            (
                (if spec.is_some_and(|s| s.text.is_some()) {
                    7.0 * scale
                } else {
                    (width as f32 - size) / 2.0
                }),
                (height as f32 - if spec.is_some() { size } else { 8.0 * scale }) / 2.0,
            ),
        )?;
        let brush = p.target.CreateSolidColorBrush(&color(ink, 1.0), None)?;
        p.begin(
            item.hDC,
            item.rcItem,
            if spec.is_some() { background } else { 0x0a0d12 },
        )?;
        if spec.is_some() {
            brush.SetColor(&color(
                if active {
                    mix(0x333a44, 0xd9ff38, 0.42)
                } else {
                    0x343c46
                },
                if enabled { 1.0 } else { 0.45 },
            ));
            p.target.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: 0.5 * scale,
                        top: 0.5 * scale,
                        right: width as f32 - 0.5 * scale,
                        bottom: height as f32 - 0.5 * scale,
                    },
                    radiusX: 3.0 * scale,
                    radiusY: 3.0 * scale,
                },
                &brush,
                scale,
                None,
            );
        }
        icon.draw(p, &brush, ink, if enabled { 1.0 } else { 0.45 });
        if focused {
            brush.SetColor(&color(0xd9ff38, 1.0));
            p.target.DrawRectangle(
                &D2D_RECT_F {
                    left: 2.0 * scale,
                    top: 2.0 * scale,
                    right: width as f32 - 2.0 * scale,
                    bottom: height as f32 - 2.0 * scale,
                },
                &brush,
                scale,
                None,
            );
        }
        p.target.EndDraw(None, None)?;
        if let Some(text) = spec.and_then(|s| s.text.as_deref()) {
            if let Some((_, font)) = self.font {
                let old = SelectObject(item.hDC, HGDIOBJ(font.0));
                SetBkMode(item.hDC, TRANSPARENT);
                SetTextColor(
                    item.hDC,
                    COLORREF(if enabled { 0xe1d5cb } else { 0x7a7269 }),
                );
                let mut rect = item.rcItem;
                rect.left += px(31, dpi);
                DrawTextW(
                    item.hDC,
                    &mut text.encode_utf16().collect::<Vec<_>>(),
                    &mut rect,
                    DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
                );
                SelectObject(item.hDC, old);
            }
        }
        Ok(())
    }
}
impl Drop for Paint {
    fn drop(&mut self) {
        unsafe {
            if let Some((_, font)) = self.font.take() {
                let _ = DeleteObject(HGDIOBJ(font.0));
            }
            let _ = DeleteObject(HGDIOBJ(self.background.0));
        }
    }
}
