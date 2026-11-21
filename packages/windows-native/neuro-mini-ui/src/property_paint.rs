//! 28 DIP 属性控件、色块及数值图标共用主题绘图器；真实 EDIT 保留输入法和系统键盘。
use super::*;
use crate::paint::{color, mix};
use windows::Win32::Graphics::Direct2D::{Common::*, *};
pub(super) fn rgb(value: u32) -> COLORREF {
    COLORREF(((value & 255) << 16) | (value & 0xff00) | ((value >> 16) & 255))
}
pub(super) unsafe fn shell(d: &Data, hwnd: HWND, dc: HDC) -> Result<()> {
    let p = &d.painter;
    let s = d.dpi.get() as f32 / 96.0;
    let mut bounds = RECT {
        left: 0,
        top: 0,
        right: d.width.get(),
        bottom: d.height.get(),
    };
    if d.stack_width.is_none() || hwnd != d.viewport.get() {
        p.shell(
            dc,
            d.width.get(),
            d.height.get(),
            d.dpi.get(),
            &[],
            d.join.get(),
        )?;
        if d.stack_width.is_some() {
            scroll::paint(d, dc)?;
            return Ok(());
        }
    } else {
        GetClientRect(hwnd, &mut bounds)?;
        p.begin(dc, bounds, p.palette.surface)?;
        p.target.EndDraw(None, None)?;
    }
    let brush = p.target.CreateSolidColorBrush(
        &color(mix(p.palette.control, p.palette.text, 0.06), 1.0),
        None,
    )?;
    // 图标资源也在 BeginDraw 前准备，出错不会遗留一个未结束的绘制事务。
    let icons = d
        .controls
        .borrow()
        .iter()
        .filter(|c| {
            matches!(
                c.field.kind,
                PropertyKind::Number { .. }
                    | PropertyKind::ReadOnly { .. }
                    | PropertyKind::Ratio { .. }
            )
        })
        .map(|c| {
            let mut frame = c.frame;
            frame.top -= d.scroll_top.get().round() as i32;
            frame.bottom -= d.scroll_top.get().round() as i32;
            Ok((
                frame,
                c.field
                    .icon
                    .filter(|_| c.ratio.is_none())
                    .map(|icon| {
                        crate::icon::prepare(
                            p,
                            icon,
                            14.0 * s / 24.0,
                            (frame.left as f32 + 4.0 * s, frame.top as f32 + 7.0 * s),
                        )
                    })
                    .transpose()?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    // 两次绑定使用同一 viewport 尺寸，避免重分配 DCRenderTarget 后丢失已清理的底色。
    p.target.BindDC(dc, &bounds)?;
    p.target.BeginDraw();
    for (frame, icon) in icons {
        brush.SetColor(&color(mix(p.palette.control, p.palette.text, 0.06), 1.0));
        p.target.FillRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: frame.left as f32,
                    top: frame.top as f32,
                    right: frame.right as f32,
                    bottom: frame.bottom as f32,
                },
                radiusX: 4.0 * s,
                radiusY: 4.0 * s,
            },
            &brush,
        );
        if let Some(icon) = icon {
            icon.draw(p, &brush, p.palette.muted, 1.0);
        }
    }
    brush.SetColor(&color(mix(p.palette.surface, p.palette.text, 0.12), 1.0));
    for c in d
        .controls
        .borrow()
        .iter()
        .filter(|c| d.section_starts.borrow().contains(&c.field.id))
    {
        let y = c.frame.top as f32 - d.scroll_top.get() as f32 - 4.0 * s;
        p.target.DrawLine(
            D2D_POINT_2F { x: 0.0, y },
            D2D_POINT_2F {
                x: bounds.right as f32,
                y,
            },
            &brush,
            s,
            None,
        );
    }
    p.target.EndDraw(None, None)?;
    let old = SelectObject(dc, HGDIOBJ(d.small_font.get().0));
    SetBkMode(dc, TRANSPARENT);
    SetTextColor(dc, rgb(p.palette.muted));
    for c in d.controls.borrow().iter() {
        if let PropertyKind::Ratio { active, .. } = c.field.kind {
            let mut rect = c.frame;
            rect.top -= d.scroll_top.get().round() as i32;
            rect.bottom -= d.scroll_top.get().round() as i32;
            rect.left += crate::layout::px(54, d.dpi.get());
            rect.right = rect.left + crate::layout::px(6, d.dpi.get());
            SetTextColor(
                dc,
                rgb(if active {
                    p.palette.signal
                } else {
                    p.palette.text
                }),
            );
            DrawTextW(
                dc,
                &mut [b':' as u16],
                &mut rect,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER | DT_NOPREFIX,
            );
            SetTextColor(dc, rgb(p.palette.muted));
        }
        if let PropertyKind::Number { suffix, .. } = c.field.kind {
            if suffix.is_empty() {
                continue;
            }
            let mut rect = c.frame;
            rect.top -= d.scroll_top.get().round() as i32;
            rect.bottom -= d.scroll_top.get().round() as i32;
            rect.right -= crate::layout::px(14, d.dpi.get());
            rect.left = rect.right - layout::suffix_width(d, &c.field.kind);
            DrawTextW(
                dc,
                &mut suffix.encode_utf16().collect::<Vec<_>>(),
                &mut rect,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER | DT_NOPREFIX,
            );
        }
    }
    SelectObject(dc, old);
    Ok(())
}
pub(super) unsafe fn control(d: &Data, draw: &DRAWITEMSTRUCT) -> Result<()> {
    let fields = d.controls.borrow();
    let Some(c) = fields
        .iter()
        .find(|c| c.handles().any(|h| h == draw.hwndItem))
    else {
        return Ok(());
    };
    let p = &d.painter;
    let s = d.dpi.get() as f32 / 96.0;
    let width = (draw.rcItem.right - draw.rcItem.left) as f32;
    let height = (draw.rcItem.bottom - draw.rcItem.top) as f32;
    let label = matches!(c.field.kind, PropertyKind::Label { .. });
    let compact = d.stack_width.is_some()
        && matches!(&c.field.kind, PropertyKind::Action { value, width, .. } if *width < 48 && !value.is_empty());
    let icon_only = d.stack_width.is_some()
        && matches!(&c.field.kind, PropertyKind::Action { value, .. } if value.is_empty());
    let active = matches!(
        c.field.kind,
        PropertyKind::Action { active: true, .. } | PropertyKind::Ratio { active: true, .. }
    );
    let base = mix(p.palette.control, p.palette.text, 0.06);
    let bg = if label {
        p.palette.surface
    } else if active {
        mix(base, p.palette.signal, 0.12)
    } else if draw.itemState.0 & ODS_SELECTED.0 != 0 {
        mix(base, p.palette.text, 0.06)
    } else {
        base
    };
    let brush = p
        .target
        .CreateSolidColorBrush(&color(p.palette.text, 1.0), None)?;
    let icon = if c.hwnd == draw.hwndItem {
        c.field
            .icon
            .map(|i| {
                if c.ratio.is_some() {
                    crate::icon::prepare(p, i, 13.5 * s / 24.0, (0.75 * s, 7.25 * s))
                } else if icon_only {
                    crate::icon::prepare(p, i, 18.0 * s / 24.0, ((width - 18.0 * s) / 2.0, 5.0 * s))
                } else if label || compact {
                    crate::icon::prepare(p, i, 14.0 * s / 24.0, (2.0 * s, 7.0 * s))
                } else {
                    crate::icon::prepare(p, i, 14.0 * s / 24.0, (7.0 * s, 7.0 * s))
                }
            })
            .transpose()?
    } else {
        None
    };
    let rounded = d.stack_width.is_some() && c.hwnd == draw.hwndItem && !label;
    p.begin(
        draw.hDC,
        draw.rcItem,
        if rounded { p.palette.surface } else { bg },
    )?;
    let face = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F {
            left: 0.5 * s,
            top: 0.5 * s,
            right: width - 0.5 * s,
            bottom: height - 0.5 * s,
        },
        radiusX: 4.0 * s,
        radiusY: 4.0 * s,
    };
    if rounded {
        brush.SetColor(&color(bg, 1.0));
        p.target.FillRoundedRectangle(&face, &brush);
    }
    let ink = if active {
        p.palette.signal
    } else if IsWindowEnabled(draw.hwndItem).as_bool() {
        p.palette.text
    } else {
        p.palette.muted
    };
    if let Some(icon) = icon {
        icon.draw(p, &brush, ink, 1.0);
    }
    if let PropertyKind::Color {
        rgb: color_value,
        stroke,
    } = c.field.kind
    {
        let rect = D2D_RECT_F {
            left: 6.0 * s,
            top: 6.0 * s,
            right: width - 6.0 * s,
            bottom: height - 6.0 * s,
        };
        brush.SetColor(&color(color_value.unwrap_or(p.palette.muted), 1.0));
        if stroke {
            p.target.DrawRectangle(&rect, &brush, 3.0 * s, None);
        } else {
            p.target.FillRectangle(&rect, &brush);
        }
        if color_value.is_none() {
            brush.SetColor(&color(0xf43f5e, 1.0));
            p.target.DrawLine(
                D2D_POINT_2F {
                    x: 4.0 * s,
                    y: height - 4.0 * s,
                },
                D2D_POINT_2F {
                    x: width - 4.0 * s,
                    y: 4.0 * s,
                },
                &brush,
                1.8 * s,
                None,
            );
        }
    }
    if (active && c.ratio.is_none()) || draw.itemState.0 & ODS_FOCUS.0 != 0 {
        brush.SetColor(&color(p.palette.signal, 1.0));
        if rounded {
            p.target.DrawRoundedRectangle(&face, &brush, s, None);
        } else {
            p.target.DrawRectangle(
                &D2D_RECT_F {
                    left: 0.5 * s,
                    top: 0.5 * s,
                    right: width - 0.5 * s,
                    bottom: height - 0.5 * s,
                },
                &brush,
                s,
                None,
            );
        }
    }
    p.target.EndDraw(None, None)?;
    let ratio_step = c
        .ratio
        .as_ref()
        .and_then(|r| r.handles().skip(2).position(|h| h == draw.hwndItem));
    let text = if c.step[0] == draw.hwndItem || ratio_step.is_some_and(|i| i % 2 == 0) {
        "▴"
    } else if c.step[1] == draw.hwndItem || ratio_step.is_some_and(|i| i % 2 == 1) {
        "▾"
    } else if let PropertyKind::Action { value, .. }
    | PropertyKind::Dropdown { value, .. }
    | PropertyKind::Label { value, .. } = &c.field.kind
    {
        value
    } else {
        ""
    };
    if !text.is_empty() {
        let dropdown = matches!(c.field.kind, PropertyKind::Dropdown { .. });
        let old = SelectObject(
            draw.hDC,
            HGDIOBJ(if dropdown || compact || label {
                d.small_font.get().0
            } else {
                d.font.get().0
            }),
        );
        SetBkMode(draw.hDC, TRANSPARENT);
        SetTextColor(draw.hDC, rgb(ink));
        let mut text: Vec<_> = text.encode_utf16().collect();
        let mut rect = text_frame(
            draw.rcItem,
            c.hwnd == draw.hwndItem && c.field.icon.is_some(),
            dropdown,
            s,
        );
        if compact || label {
            rect.left = draw.rcItem.left
                + ((if c.field.icon.is_some() { 18.0 } else { 2.0 }) * s).round() as i32;
            rect.right = draw.rcItem.right - (2.0 * s).round() as i32;
        }
        DrawTextW(
            draw.hDC,
            &mut text,
            &mut rect,
            DT_SINGLELINE
                | DT_VCENTER
                | if label || (dropdown && c.field.icon.is_some()) {
                    DT_LEFT
                } else {
                    DT_CENTER
                }
                | DT_NOPREFIX
                | DT_END_ELLIPSIS,
        );
        if dropdown {
            let mut rect = draw.rcItem;
            rect.left = rect.right - (14.0 * s).round() as i32;
            DrawTextW(
                draw.hDC,
                &mut if c.expanded { "▴" } else { "▾" }
                    .encode_utf16()
                    .collect::<Vec<_>>(),
                &mut rect,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER | DT_NOPREFIX,
            );
        }
        SelectObject(draw.hDC, old);
    }
    Ok(())
}
fn text_frame(mut rect: RECT, icon: bool, dropdown: bool, scale: f32) -> RECT {
    // 图标区和文字区不可重叠；只有下拉 trigger 还需为右侧箭头保留空间。
    if icon || dropdown {
        rect.left += ((if icon { 25.0 } else { 4.0 }) * scale).round() as i32;
        rect.right -= ((if dropdown { 14.0 } else { 4.0 }) * scale).round() as i32;
    }
    rect
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn icon_labels_reserve_separate_space_without_changing_plain_or_dropdown_geometry() {
        for scale in [1.0, 1.5, 2.0] {
            let rect = RECT {
                left: 0,
                top: 0,
                right: (55.0 * scale) as i32,
                bottom: (28.0 * scale) as i32,
            };
            for (icon, dropdown, left, right) in [
                (false, false, 0., 0.),
                (true, false, 25., 4.),
                (true, true, 25., 14.),
                (false, true, 4., 14.),
            ] {
                let result = text_frame(rect, icon, dropdown, scale);
                assert_eq!(result.left, (left * scale).round() as i32);
                assert_eq!(result.right, rect.right - (right * scale).round() as i32);
                assert_eq!((result.top, result.bottom), (rect.top, rect.bottom));
                assert!(result.right > result.left);
            }
        }
    }
}
