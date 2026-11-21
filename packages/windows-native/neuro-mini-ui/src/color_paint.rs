//! 有界 GDI 渐变缓冲和主题图标；整块背景离屏完成后一次复制，不擦白底、不读取屏幕内容。
use super::*;
use crate::paint::{color, mix};
pub(super) fn rgb(v: u32) -> COLORREF {
    COLORREF(((v & 255) << 16) | (v & 0xff00) | ((v >> 16) & 255))
}
fn packed(c: Rgba) -> u32 {
    (c.r as u32) << 16 | (c.g as u32) << 8 | c.b as u32
}
fn px(d: &Data, r: RECT) -> RECT {
    RECT {
        left: crate::layout::px(r.left, d.dpi),
        top: crate::layout::px(r.top, d.dpi),
        right: crate::layout::px(r.right, d.dpi),
        bottom: crate::layout::px(r.bottom, d.dpi),
    }
}
unsafe fn fill(dc: HDC, r: RECT, c: u32) {
    SetDCBrushColor(dc, rgb(c));
    FillRect(dc, &r, HBRUSH(GetStockObject(DC_BRUSH).0));
}
unsafe fn border(dc: HDC, r: RECT, c: u32) {
    SetDCBrushColor(dc, rgb(c));
    FrameRect(dc, &r, HBRUSH(GetStockObject(DC_BRUSH).0));
}
unsafe fn pixels(
    dc: HDC,
    r: RECT,
    w: i32,
    h: i32,
    mut get: impl FnMut(i32, i32) -> u32,
) -> Result<()> {
    let data: Vec<u32> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| get(x, y))
        .collect();
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let copied = StretchDIBits(
        dc,
        r.left,
        r.top,
        r.right - r.left,
        r.bottom - r.top,
        0,
        0,
        w,
        h,
        Some(data.as_ptr().cast()),
        &info,
        DIB_RGB_COLORS,
        SRCCOPY,
    );
    if copied == 0 || copied == GDI_ERROR {
        return Err(Error::new(E_FAIL, "Color gradient StretchDIBits failed"));
    }
    Ok(())
}
unsafe fn patch(dc: HDC, r: RECT, c: Rgba) -> Result<()> {
    pixels(dc, r, r.right - r.left, r.bottom - r.top, |x, y| {
        let base = if (x / 4 + y / 4) % 2 == 0 {
            0x999999
        } else {
            0xdddddd
        };
        mix(base, packed(c), c.a as f32 / 255.0)
    })
}
unsafe fn text(d: &Data, dc: HDC, r: RECT, value: &str, ink: u32) {
    let old = SelectObject(dc, HGDIOBJ(d.font.0));
    SetBkMode(dc, TRANSPARENT);
    SetTextColor(dc, rgb(ink));
    let mut value: Vec<u16> = value.encode_utf16().collect();
    let mut r = r;
    DrawTextW(
        dc,
        &mut value,
        &mut r,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
    );
    SelectObject(dc, old);
}
struct Buffer {
    dc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
}
impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            let _ = DeleteObject(HGDIOBJ(self.bitmap.0));
            let _ = DeleteDC(self.dc);
        }
    }
}
impl Buffer {
    unsafe fn new(dc: HDC, w: i32, h: i32) -> Result<Self> {
        let mem = CreateCompatibleDC(Some(dc));
        if mem.is_invalid() {
            return Err(Error::from_win32());
        }
        let bitmap = CreateCompatibleBitmap(dc, w, h);
        if bitmap.is_invalid() {
            let _ = DeleteDC(mem);
            return Err(Error::from_win32());
        }
        let old = SelectObject(mem, HGDIOBJ(bitmap.0));
        Ok(Self {
            dc: mem,
            bitmap,
            old,
        })
    }
}
pub(super) unsafe fn shell(d: &Data, dc: HDC) -> Result<()> {
    let width = crate::layout::px(geometry::WIDTH, d.dpi);
    let height = crate::layout::px(geometry::HEIGHT, d.dpi);
    let buffer = Buffer::new(dc, width, height)?;
    let out = buffer.dc;
    let v = d.value.get();
    let p = d.painter.palette;
    fill(out, geometry::rect(0, 0, width, height), p.surface);
    border(
        out,
        geometry::rect(0, 0, width, height),
        mix(p.surface, 0xffffff, 0.12),
    );
    pixels(out, px(d, geometry::SV), 230, 108, |x, y| {
        packed(
            value::Hsva {
                h: v.h,
                s: x as f32 / 229.0,
                v: 1.0 - y as f32 / 107.0,
                a: 1.0,
            }
            .color(),
        )
    })?;
    pixels(out, px(d, geometry::HUE), 230, 12, |x, _| {
        packed(
            value::Hsva {
                h: x as f32 / 229.0 * 360.0,
                s: 1.0,
                v: 1.0,
                a: 1.0,
            }
            .color(),
        )
    })?;
    let current = packed(v.color());
    pixels(out, px(d, geometry::ALPHA), 230, 12, |x, y| {
        mix(
            if (x / 4 + y / 4) % 2 == 0 {
                0x999999
            } else {
                0xdddddd
            },
            current,
            x as f32 / 229.0,
        )
    })?;
    let s = d.dpi as f32 / 96.0;
    let radius = (6.0 * s).round() as i32;
    let x = ((geometry::SV.left as f32 + v.s * 230.0) * s).round() as i32;
    let y = ((geometry::SV.top as f32 + (1.0 - v.v) * 108.0) * s).round() as i32;
    let pen = CreatePen(PS_SOLID, crate::layout::px(2, d.dpi), rgb(0xffffff));
    let old = SelectObject(out, HGDIOBJ(pen.0));
    let brush = SelectObject(out, GetStockObject(NULL_BRUSH));
    let _ = Ellipse(out, x - radius, y - radius, x + radius, y + radius);
    SelectObject(out, brush);
    SelectObject(out, old);
    let _ = DeleteObject(HGDIOBJ(pen.0));
    for (rect, n) in [(geometry::HUE, v.h / 360.0), (geometry::ALPHA, v.a)] {
        let x = rect.left as f32 + (rect.right - rect.left) as f32 * n;
        border(
            out,
            px(d, geometry::rect(x.round() as i32 - 2, rect.top - 2, 5, 16)),
            0xffffff,
        );
    }
    for y in [165, 205, 276] {
        fill(
            out,
            px(d, geometry::rect(9, y, 230, 1)),
            mix(p.surface, 0xffffff, 0.12),
        );
    }
    text(
        d,
        out,
        px(d, geometry::rect(229, 173, 10, 26)),
        "%",
        p.muted,
    );
    BitBlt(
        dc,
        0,
        0,
        d.width,
        d.height,
        Some(out),
        d.origin.get().x,
        d.origin.get().y,
        SRCCOPY,
    )?;
    Ok(())
}
unsafe fn icon(d: &Data, dc: HDC, r: RECT, i: Icon, ink: u32) -> Result<()> {
    let p = &d.painter;
    let s = d.dpi as f32 / 96.0;
    let size = (18.0 * s)
        .min((r.right - r.left - 2).max(1) as f32)
        .min((r.bottom - r.top - 2).max(1) as f32);
    let shape = crate::icon::prepare(
        p,
        i,
        size / 24.0,
        (
            ((r.right - r.left) as f32 - size) / 2.0,
            ((r.bottom - r.top) as f32 - size) / 2.0,
        ),
    )?;
    let brush = p.target.CreateSolidColorBrush(&color(ink, 1.0), None)?;
    p.begin(dc, r, p.palette.control)?;
    shape.draw(p, &brush, ink, 1.0);
    p.target.EndDraw(None, None)
}
pub(super) unsafe fn palette(d: &Data, dc: HDC, focus: bool) -> Result<()> {
    let p = d.painter.palette;
    let frame = px(d, geometry::PALETTE);
    fill(
        dc,
        geometry::rect(0, 0, frame.right - frame.left, frame.bottom - frame.top),
        p.surface,
    );
    let colors = d.palette.borrow();
    for i in 0..=colors.len() {
        let r = geometry::swatch(i, d.scroll.get());
        if r.bottom < geometry::PALETTE.top || r.top > geometry::PALETTE.bottom {
            continue;
        }
        let local = |mut r: RECT| {
            r.left -= frame.left;
            r.right -= frame.left;
            r.top -= frame.top;
            r.bottom -= frame.top;
            r
        };
        let rect = local(px(d, r));
        if i == 0 {
            fill(dc, rect, p.control);
            text(d, dc, rect, "+", p.text);
        } else {
            patch(dc, rect, colors[i - 1])?;
            border(
                dc,
                rect,
                if d.selected.get() == Some(i - 1) {
                    p.signal
                } else {
                    p.muted
                },
            );
            let remove = local(px(d, geometry::rect(r.right - 6, r.top - 5, 13, 13)));
            icon(d, dc, remove, d.icons.remove, p.muted)?;
        }
    }
    if focus {
        border(
            dc,
            geometry::rect(0, 0, frame.right - frame.left, frame.bottom - frame.top),
            p.signal,
        );
    }
    Ok(())
}
pub(super) unsafe fn control(d: &Data, draw: &DRAWITEMSTRUCT) -> Result<()> {
    let id = draw.CtlID as i32;
    let p = d.painter.palette;
    let focus = draw.itemState.0 & ODS_FOCUS.0 != 0;
    let r = draw.rcItem;
    if id == controls::PALETTE {
        return palette(d, draw.hDC, focus);
    }
    fill(draw.hDC, r, p.control);
    match id {
        controls::PREVIEW => patch(draw.hDC, r, d.value.get().color())?,
        controls::PICK => icon(
            d,
            draw.hDC,
            r,
            d.icons.eyedropper,
            if d.screen_pick { p.text } else { p.muted },
        )?,
        controls::APPLY => {
            fill(draw.hDC, r, mix(p.control, p.signal, 0.08));
            icon(
                d,
                draw.hDC,
                geometry::rect(0, 0, crate::layout::px(25, d.dpi), r.bottom - r.top),
                d.icons.apply,
                p.signal,
            )?;
            let mut label = r;
            label.left += crate::layout::px(24, d.dpi);
            text(d, draw.hDC, label, "应用", p.signal);
        }
        controls::CANCEL => text(d, draw.hDC, r, "取消", p.text),
        _ => (),
    }
    border(
        draw.hDC,
        r,
        if focus || id == controls::APPLY {
            p.signal
        } else {
            mix(p.surface, 0xffffff, 0.12)
        },
    );
    Ok(())
}
