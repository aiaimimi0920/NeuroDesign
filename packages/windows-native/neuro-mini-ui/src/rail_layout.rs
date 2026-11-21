//! 固定行与局部滚动行共用的胶囊几何；offset 为 DIP，不引入动画或业务状态。
use crate::{layout::px, Frame};

#[derive(Clone, Copy, Debug)]
pub struct RailLayout {
    pub viewport: Frame,
    pub offset: f64,
    pub max_scroll: f64,
    fixed: usize,
    rows: usize,
    top: i32,
    dpi: u32,
}
impl RailLayout {
    pub fn new(frame: Frame, dpi: u32, fixed: usize, rows: usize, offset: f64) -> Self {
        let dpi = dpi.clamp(96, 768);
        let content = (rows.saturating_mul(36).saturating_sub(8)) as f64;
        let height = frame.height.max(0).min(px(360, dpi));
        let max_scroll = (content - f64::from(height) * 96.0 / f64::from(dpi)).max(0.0);
        Self {
            viewport: Frame {
                y: frame.y.saturating_add(px((fixed * 36) as i32, dpi)),
                height: height.min(px(content as i32, dpi)),
                ..frame
            },
            offset: if offset.is_finite() {
                offset.clamp(0.0, max_scroll)
            } else {
                0.0
            },
            max_scroll,
            fixed,
            rows,
            top: frame.y,
            dpi,
        }
    }
    pub fn row(self, index: usize) -> Option<Frame> {
        if index >= self.fixed + self.rows {
            return None;
        }
        let scroll = if index >= self.fixed {
            self.offset
        } else {
            0.0
        };
        Some(Frame {
            y: self.top.saturating_add(
                ((index as f64 * 36.0 - scroll) * f64::from(self.dpi) / 96.0).round() as i32,
            ),
            height: px(28, self.dpi),
            ..self.viewport
        })
    }
    pub fn visible(self, index: usize) -> Option<Frame> {
        let mut row = self.row(index)?;
        if index >= self.fixed {
            let bottom = (row.y + row.height).min(self.viewport.y + self.viewport.height);
            row.y = row.y.max(self.viewport.y);
            row.height = bottom - row.y;
        }
        (row.height > 0).then_some(row)
    }
    pub fn track(self) -> Option<Frame> {
        let inset = px(12, self.dpi);
        (self.max_scroll > 0.0 && self.viewport.height > inset * 2).then_some(Frame {
            x: self.viewport.x + self.viewport.width - px(18, self.dpi),
            y: self.viewport.y + inset,
            width: px(4, self.dpi),
            height: self.viewport.height - inset * 2,
        })
    }
    pub fn thumb(self) -> Option<Frame> {
        let track = self.track()?;
        let content =
            self.max_scroll * f64::from(self.dpi) / 96.0 + f64::from(self.viewport.height);
        let height =
            (f64::from(self.viewport.height) / content * f64::from(track.height)).round() as i32;
        let height = height.max(px(18, self.dpi)).min(track.height);
        Some(Frame {
            y: track.y
                + (self.offset / self.max_scroll * f64::from(track.height - height)).round() as i32,
            height,
            ..track
        })
    }
    pub fn scroll(self, delta: f64) -> f64 {
        (self.offset + delta).clamp(0.0, self.max_scroll)
    }
    /// Track 按起点比例直接跳转；thumb 拖动由调用方冻结开始时的 max/travel。
    pub fn track_offset(self, y: i32) -> f64 {
        self.track().map_or(self.offset, |track| {
            (f64::from(y - track.y) / f64::from(track.height)).clamp(0.0, 1.0) * self.max_scroll
        })
    }
    /// # Safety
    /// dc 必须是调用线程拥有的有效绘图上下文；origin 为该 DC 的屏幕原点。
    pub unsafe fn paint_scrollbar(
        self,
        dc: windows::Win32::Graphics::Gdi::HDC,
        origin: (i32, i32),
    ) {
        use windows::Win32::{
            Foundation::{COLORREF, RECT},
            Graphics::Gdi::*,
        };
        for (frame, color) in [(self.track(), 0x443a33), (self.thumb(), 0x38ffd9)] {
            if let Some(f) = frame {
                SetDCBrushColor(dc, COLORREF(color));
                FillRect(
                    dc,
                    &RECT {
                        left: f.x - origin.0,
                        top: f.y - origin.1,
                        right: f.x + f.width - origin.0,
                        bottom: f.y + f.height - origin.1,
                    },
                    HBRUSH(GetStockObject(DC_BRUSH).0),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rail(offset: f64, dpi: u32) -> RailLayout {
        RailLayout::new(
            Frame {
                x: -256,
                y: 22,
                width: 256,
                height: px(180, dpi),
            },
            dpi,
            2,
            20,
            offset,
        )
    }
    #[test]
    fn fixed_rows_stay_put_and_scrolled_rows_share_clipping() {
        let a = rail(0.0, 96);
        let b = rail(13.5, 96);
        assert_eq!(a.row(0), b.row(0));
        assert_eq!(a.row(1), b.row(1));
        assert_eq!(b.row(2).unwrap().y, 81);
        assert_eq!(b.visible(2).unwrap().y, 94);
        assert!(b.visible(19).is_none());
        assert_eq!(b.scroll(-1000.0), 0.0);
        assert_eq!(b.scroll(1000.0), b.max_scroll);
    }
    #[test]
    fn thumb_and_track_follow_baseline_at_each_dpi() {
        for dpi in [96, 120, 144, 192] {
            let r = rail(f64::MAX, dpi);
            let t = r.track().unwrap();
            let thumb = r.thumb().unwrap();
            assert_eq!(thumb.y + thumb.height, t.y + t.height);
            assert_eq!(r.track_offset(t.y), 0.0);
            assert_eq!(r.track_offset(t.y + t.height), r.max_scroll);
            assert!(thumb.height >= px(18, dpi));
        }
        assert_eq!(rail(f64::NAN, 96).offset, 0.0);
    }
}
