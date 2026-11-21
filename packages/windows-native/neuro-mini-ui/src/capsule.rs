//! 通用胶囊绘制器只拥有软件绘图和字体；图标、名称、禁用和焦点来自调用方。
use crate::{
    capsule_layout::*,
    layout::px,
    paint::{color, Painter},
    Icon,
};
use windows::{
    core::{w, Error, Result},
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            Gdi::*,
        },
    },
};
pub struct CapsulePainter {
    painter: Painter,
    font: Option<(u32, HFONT)>,
}
pub struct CapsuleAppearance<'a> {
    pub icon: Icon,
    pub label: &'a str,
    pub enabled: bool,
    pub focused: bool,
    pub action: Option<CapsuleAction>,
    /// 不透明 RGB 底色；色键宿主可传其已声明的透明色。
    pub background: u32,
}
#[derive(Clone, Copy)]
pub struct CapsuleAction {
    pub icon: Icon,
    pub active: bool,
    pub pressed: bool,
}
impl CapsulePainter {
    pub fn new() -> Result<Self> {
        Ok(Self {
            painter: unsafe { Painter::new()? },
            font: None,
        })
    }
    /// # Safety
    /// dc 为调用线程的有效 DC；宿主负责组的可见裁剪、层序和无输入背景，组件不创建窗口。
    pub unsafe fn backdrop(&mut self, dc: HDC, frame: crate::Frame, dpi: u32) -> Result<()> {
        // 只填圆角内的像素；Clear 整个绑定矩形会擦掉另一组从透明圆角透出的底色。
        let brush = SelectObject(dc, GetStockObject(DC_BRUSH));
        let pen = SelectObject(dc, GetStockObject(NULL_PEN));
        let color = SetDCBrushColor(dc, COLORREF(0x120d0a));
        let diameter = px(28, dpi);
        let result = RoundRect(
            dc,
            frame.x,
            frame.y,
            frame.x + frame.width,
            frame.y + frame.height,
            diameter,
            diameter,
        )
        .ok();
        SetDCBrushColor(dc, color);
        SelectObject(dc, pen);
        SelectObject(dc, brush);
        result
    }
    unsafe fn font(&mut self, dpi: u32) -> Result<HFONT> {
        if let Some((_, font)) = self.font.filter(|(previous, _)| *previous == dpi) {
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
    /// # Safety
    /// dc 必须属于当前 UI 线程，layout 使用该 DC 的像素坐标；绘制不改变 HWND 或输入状态。
    pub unsafe fn paint(
        &mut self,
        dc: HDC,
        layout: CapsuleLayout,
        dpi: u32,
        appearance: CapsuleAppearance<'_>,
    ) -> Result<()> {
        let font = self.font(dpi)?;
        let p = &self.painter;
        let f = layout.frame;
        let scale = dpi.max(96) as f32 / 96.0;
        let diameter = f.height as f32;
        let (x, y) = (
            (layout.anchor.x - f.x) as f32,
            (layout.anchor.y - f.y) as f32,
        );
        let icon_size = diameter * 24.0 / 28.0;
        let icon = crate::icon::prepare(
            p,
            appearance.icon,
            icon_size / 24.0,
            (
                x + (diameter - icon_size) / 2.0,
                y + (diameter - icon_size) / 2.0,
            ),
        )?;
        let action = appearance
            .action
            .zip(layout.inline_action(dpi))
            .map(|(action, frame)| {
                let icon = crate::icon::prepare(
                    p,
                    action.icon,
                    scale * 0.5,
                    (
                        (frame.x - f.x) as f32 + 2.0 * scale,
                        (frame.y - f.y) as f32 + 2.0 * scale,
                    ),
                )?;
                Ok::<_, Error>((action, frame, icon))
            })
            .transpose()?;
        let brush = p
            .target
            .CreateSolidColorBrush(&color(0x0a0d12, 1.0), None)?;
        p.begin(
            dc,
            RECT {
                left: f.x,
                top: f.y,
                right: f.x + f.width,
                bottom: f.y + f.height,
            },
            appearance.background,
        )?;
        if layout.expanded {
            let pill = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: 0.5 * scale,
                    top: 0.5 * scale,
                    right: f.width as f32 - 0.5 * scale,
                    bottom: diameter - 0.5 * scale,
                },
                radiusX: diameter / 2.0,
                radiusY: diameter / 2.0,
            };
            p.target.FillRoundedRectangle(&pill, &brush);
            brush.SetColor(&color(0x333a44, 1.0));
            p.target.DrawRoundedRectangle(&pill, &brush, scale, None);
        }
        if !layout.expanded {
            let half = x + diameter / 2.0;
            let (left, right) = if layout.side == CapsuleSide::Input {
                (x, half)
            } else {
                (half, x + diameter)
            };
            p.target.PushAxisAlignedClip(
                &D2D_RECT_F {
                    left,
                    top: y,
                    right,
                    bottom: y + diameter,
                },
                D2D1_ANTIALIAS_MODE_ALIASED,
            );
        }
        icon.draw(
            p,
            &brush,
            p.palette.text,
            if appearance.enabled { 1.0 } else { 0.4 },
        );
        if !layout.expanded {
            p.target.PopAxisAlignedClip();
        }
        if appearance.focused {
            brush.SetColor(&color(p.palette.signal, 1.0));
            p.target.DrawEllipse(
                &D2D1_ELLIPSE {
                    point: D2D_POINT_2F {
                        x: x + diameter / 2.0,
                        y: y + diameter / 2.0,
                    },
                    radiusX: (diameter / 2.0 - 2.0 * scale).max(1.0),
                    radiusY: (diameter / 2.0 - 2.0 * scale).max(1.0),
                },
                &brush,
                scale,
                None,
            );
        }
        if let Some((action, frame, icon)) = action {
            if action.pressed {
                brush.SetColor(&color(0x303943, 1.0));
                p.target.FillRectangle(
                    &D2D_RECT_F {
                        left: (frame.x - f.x) as f32,
                        top: (frame.y - f.y) as f32,
                        right: (frame.x + frame.width - f.x) as f32,
                        bottom: (frame.y + frame.height - f.y) as f32,
                    },
                    &brush,
                );
            }
            icon.draw(
                p,
                &brush,
                if action.active {
                    p.palette.text
                } else {
                    p.palette.muted
                },
                if appearance.enabled { 1.0 } else { 0.4 },
            );
        }
        p.target.EndDraw(None, None)?;
        if layout.expanded {
            let old = SelectObject(dc, HGDIOBJ(font.0));
            let old_mode = SetBkMode(dc, TRANSPARENT);
            let rgb = if appearance.enabled {
                0xe5e7eb
            } else {
                p.palette.muted
            };
            let old_color = SetTextColor(
                dc,
                COLORREF(((rgb & 255) << 16) | (rgb & 0xff00) | ((rgb >> 16) & 255)),
            );
            let mut text: Vec<u16> = appearance
                .label
                .chars()
                .take(128)
                .collect::<String>()
                .encode_utf16()
                .collect();
            let b = layout
                .label(dpi, appearance.action.is_some())
                .expect("expanded label");
            let mut rect = RECT {
                left: b.x,
                top: b.y,
                right: b.x + b.width,
                bottom: b.y + b.height,
            };
            DrawTextW(
                dc,
                &mut text,
                &mut rect,
                DT_SINGLELINE
                    | DT_VCENTER
                    | DT_NOPREFIX
                    | DT_END_ELLIPSIS
                    | if layout.side == CapsuleSide::Output {
                        DT_RIGHT
                    } else {
                        DT_LEFT
                    },
            );
            SetTextColor(dc, old_color);
            SetBkMode(dc, BACKGROUND_MODE(old_mode as u32));
            SelectObject(dc, old);
        }
        Ok(())
    }
}
impl Drop for CapsulePainter {
    fn drop(&mut self) {
        if let Some((_, font)) = self.font.take() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(font.0));
            }
        }
    }
}
#[cfg(test)]
#[path = "capsule_tests.rs"]
mod tests;
