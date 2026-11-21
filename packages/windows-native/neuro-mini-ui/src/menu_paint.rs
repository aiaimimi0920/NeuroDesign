//! 菜单的稳定 20 DIP 行、15 DIP 图标和快捷键列；普通组与 quiet 模式组使用独立选择态。
use super::*;
use crate::paint::{color, mix};
use windows::Win32::Graphics::Direct2D::{Common::*, *};
pub(super) unsafe fn shell(data: &Data, dc: HDC) -> Result<()> {
    let p = &data.painter;
    let s = data.dpi as f32 / 96.0;
    let background = mix(0x0e1218, p.palette.text, 0.04);
    let brush = p
        .target
        .CreateSolidColorBrush(&color(mix(p.palette.surface, 0xffffff, 0.12), 1.0), None)?;
    let (top, bottom) = if data.above {
        (0.5 * s, data.height as f32 - 6.0 * s)
    } else {
        (6.0 * s, data.height as f32 - 0.5 * s)
    };
    let tip = crate::path::geometry_at(
        &p.factory,
        if data.above {
            "M-5 0 0 5 5 0"
        } else {
            "M-5 0 0-5 5 0"
        },
        s,
        (data.pointer as f32, if data.above { bottom } else { top }),
        false,
    )?;
    p.begin(
        dc,
        RECT {
            left: 0,
            top: 0,
            right: data.width,
            bottom: data.height,
        },
        background,
    )?;
    let rect = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F {
            left: 0.5 * s,
            top,
            right: data.width as f32 - 0.5 * s,
            bottom,
        },
        radiusX: if data.quiet { 6.0 * s } else { 4.0 * s },
        radiusY: if data.quiet { 6.0 * s } else { 4.0 * s },
    };
    p.target.DrawRoundedRectangle(&rect, &brush, s, None);
    // 抹去指针下面的一段壳线，使尖角与面板是一条连续轮廓。
    brush.SetColor(&color(background, 1.0));
    p.target.DrawLine(
        D2D_POINT_2F {
            x: data.pointer as f32 - 4.0 * s,
            y: if data.above { bottom } else { top },
        },
        D2D_POINT_2F {
            x: data.pointer as f32 + 4.0 * s,
            y: if data.above { bottom } else { top },
        },
        &brush,
        2.0 * s,
        None,
    );
    brush.SetColor(&color(mix(p.palette.surface, 0xffffff, 0.12), 1.0));
    p.target.DrawGeometry(&tip, &brush, s, None);
    p.target.EndDraw(None, None)
}
pub(super) unsafe fn row(data: &Data, index: usize, draw: &DRAWITEMSTRUCT) -> Result<()> {
    let p = &data.painter;
    let item = &data.items[index];
    let s = data.dpi as f32 / 96.0;
    let hot = data.focus.get() == Some(index);
    let base = mix(0x0e1218, p.palette.text, 0.04);
    let background = if data.quiet {
        if hot {
            mix(
                mix(p.palette.control, p.palette.text, 0.06),
                p.palette.text,
                0.18,
            )
        } else if item.selected {
            mix(p.palette.control, p.palette.text, 0.06)
        } else {
            base
        }
    } else if item.selected {
        mix(base, p.palette.signal, if hot { 0.25 } else { 0.14 })
    } else if hot {
        mix(base, p.palette.text, 0.18)
    } else {
        base
    };
    let ink = if !item.enabled {
        p.palette.muted
    } else if item.selected && !data.quiet {
        p.palette.signal
    } else {
        p.palette.text
    };
    let brush = p
        .target
        .CreateSolidColorBrush(&color(ink, if item.enabled { 1.0 } else { 0.4 }), None)?;
    let shape = crate::icon::prepare(
        p,
        item.icon,
        15.0 * s / 24.0,
        (if data.quiet { 5.0 * s } else { 9.0 * s }, 2.5 * s),
    )?;
    p.begin(draw.hDC, draw.rcItem, background)?;
    if item.selected && !data.quiet {
        brush.SetColor(&color(p.palette.signal, 1.0));
        p.target.FillRectangle(
            &D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: 2.0 * s,
                bottom: 20.0 * s,
            },
            &brush,
        );
    }
    brush.SetColor(&color(ink, if item.enabled { 1.0 } else { 0.4 }));
    shape.draw(p, &brush, ink, if item.enabled { 1.0 } else { 0.4 });
    p.target.EndDraw(None, None)?;
    let old = SelectObject(draw.hDC, HGDIOBJ(data.font.0));
    SetBkMode(draw.hDC, TRANSPARENT);
    SetTextColor(draw.hDC, rgb(ink));
    let mut rect = draw.rcItem;
    rect.left += (if data.quiet { 25.0 * s } else { 29.0 * s }).round() as i32;
    rect.right -= (8.0 * s).round() as i32;
    let mut label: Vec<_> = item.label.encode_utf16().collect();
    DrawTextW(
        draw.hDC,
        &mut label,
        &mut rect,
        DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
    );
    if !item.shortcut.is_empty() {
        SetTextColor(
            draw.hDC,
            rgb(if item.selected {
                p.palette.signal
            } else {
                p.palette.muted
            }),
        );
        let mut text: Vec<_> = item.shortcut.encode_utf16().collect();
        DrawTextW(
            draw.hDC,
            &mut text,
            &mut rect,
            DT_SINGLELINE | DT_VCENTER | DT_RIGHT | DT_NOPREFIX,
        );
    }
    SelectObject(draw.hDC, old);
    Ok(())
}
fn rgb(value: u32) -> COLORREF {
    COLORREF(((value & 255) << 16) | (value & 0xff00) | ((value >> 16) & 255))
}
