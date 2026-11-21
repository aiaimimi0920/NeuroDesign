//! 28 DIP 文本行与 8 DIP 滚动条；只绘制有界可见行，选择不改变控件几何。
use super::*;
use crate::paint::{color, mix};
use windows::Win32::Graphics::Direct2D::{Common::*, *};
fn rgb(value: u32) -> COLORREF {
    COLORREF(((value & 255) << 16) | (value & 0xff00) | ((value >> 16) & 255))
}
pub(super) unsafe fn shell(d: &Data, hwnd: HWND, dc: HDC) -> Result<()> {
    let mut rect = RECT::default();
    GetClientRect(hwnd, &mut rect)?;
    let p = &d.painter;
    let s = d.frame.dpi as f32 / 96.0;
    let brush = p
        .target
        .CreateSolidColorBrush(&color(mix(p.palette.surface, 0xffffff, 0.12), 1.0), None)?;
    p.begin(dc, rect, mix(0x0e1218, p.palette.text, 0.04))?;
    if hwnd == d.root.get() {
        p.target.DrawRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: 0.5 * s,
                    top: 0.5 * s,
                    right: rect.right as f32 - 0.5 * s,
                    bottom: rect.bottom as f32 - 0.5 * s,
                },
                radiusX: 4.0 * s,
                radiusY: 4.0 * s,
            },
            &brush,
            s,
            None,
        );
        if d.frame.maximum() > 0.0 {
            let r = d.frame.thumb(d.top.get());
            brush.SetColor(&color(
                mix(
                    0x0e1218,
                    p.palette.text,
                    if d.drag.get().is_some() { 0.38 } else { 0.24 },
                ),
                1.0,
            ));
            let thumb = D2D_RECT_F {
                left: r.left as f32,
                top: r.top as f32,
                right: r.right as f32,
                bottom: r.bottom as f32,
            };
            p.target.FillRectangle(&thumb, &brush);
            if d.focus.get() == Some(d.items.len()) {
                brush.SetColor(&color(p.palette.signal, 1.0));
                p.target.DrawRectangle(&thumb, &brush, s, None);
            }
        }
    }
    p.target.EndDraw(None, None)
}
pub(super) unsafe fn row(d: &Data, draw: &DRAWITEMSTRUCT) -> Result<()> {
    let value = GetWindowLongPtrW(draw.hwndItem, GWLP_USERDATA);
    let Some(index) = value.checked_sub(1).filter(|i| *i >= 0).map(|i| i as usize) else {
        return Ok(());
    };
    let Some(label) = d.items.get(index) else {
        return Ok(());
    };
    let p = &d.painter;
    let base = mix(0x0e1218, p.palette.text, 0.04);
    let active = d.selected == Some(index);
    let hot = d.enabled[index] && d.hover.get() == Some(index);
    let focus = d.focus.get() == Some(index);
    let background = if active {
        mix(base, p.palette.signal, if hot { 0.25 } else { 0.14 })
    } else if hot {
        mix(base, p.palette.text, 0.18)
    } else {
        base
    };
    let ink = if !d.enabled[index] {
        p.palette.muted
    } else if active {
        p.palette.signal
    } else {
        p.palette.text
    };
    let brush = p
        .target
        .CreateSolidColorBrush(&color(p.palette.signal, 1.0), None)?;
    p.begin(draw.hDC, draw.rcItem, background)?;
    let s = d.frame.dpi as f32 / 96.0;
    if focus {
        p.target.DrawRectangle(
            &D2D_RECT_F {
                left: s,
                top: s,
                right: (draw.rcItem.right - draw.rcItem.left) as f32 - s,
                bottom: d.frame.row as f32 - s,
            },
            &brush,
            s,
            None,
        );
    }
    p.target.EndDraw(None, None)?;
    let old = SelectObject(draw.hDC, HGDIOBJ(d.font.0));
    SetBkMode(draw.hDC, TRANSPARENT);
    SetTextColor(draw.hDC, rgb(ink));
    let mut rect = draw.rcItem;
    rect.left += crate::layout::px(8, d.frame.dpi);
    rect.right -= crate::layout::px(8, d.frame.dpi);
    DrawTextW(
        draw.hDC,
        &mut label.encode_utf16().collect::<Vec<_>>(),
        &mut rect,
        DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
    );
    SelectObject(draw.hDC, old);
    Ok(())
}
