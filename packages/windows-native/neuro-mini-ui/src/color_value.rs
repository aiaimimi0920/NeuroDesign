//! 颜色选择器的字节边界和本地 HSV 草稿；输入尚未完整时不覆盖用户文本。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    pub fn hex(self) -> String {
        let Self { r, g, b, a } = self;
        if a == 255 {
            format!("#{r:02x}{g:02x}{b:02x}")
        } else {
            format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
        }
    }
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim().strip_prefix('#').unwrap_or(text.trim());
        if !matches!(text.len(), 6 | 8) || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let byte = |n| u8::from_str_radix(&text[n..n + 2], 16).ok();
        Some(Self {
            r: byte(0)?,
            g: byte(2)?,
            b: byte(4)?,
            a: if text.len() == 8 { byte(6)? } else { 255 },
        })
    }
    pub fn rgb_text(self) -> String {
        let Self { r, g, b, a } = self;
        if a == 255 {
            format!("rgb({r}, {g}, {b})")
        } else {
            let alpha = (a as f64 / 255.0 * 100.0).round() / 100.0;
            format!("rgba({r}, {g}, {b}, {alpha})")
        }
    }
    pub(crate) fn same_palette(self, other: Self) -> bool {
        self == other || self.a == 0 && other.a == 0
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Hsva {
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub a: f32,
}
impl Hsva {
    pub fn new(color: Rgba) -> Self {
        let mut value = Self {
            h: 0.0,
            s: 0.0,
            v: 0.0,
            a: 1.0,
        };
        value.load(color);
        value
    }
    pub fn load(&mut self, color: Rgba) {
        let [r, g, b] = [color.r, color.g, color.b].map(|v| v as f32 / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        if delta != 0.0 {
            self.h = (if max == r {
                ((g - b) / delta).rem_euclid(6.0)
            } else if max == g {
                (b - r) / delta + 2.0
            } else {
                (r - g) / delta + 4.0
            } * 60.0);
        }
        self.s = if max == 0.0 { 0.0 } else { delta / max };
        self.v = max;
        self.a = color.a as f32 / 255.0;
    }
    pub fn color(self) -> Rgba {
        let h = self.h.rem_euclid(360.0) / 60.0;
        let c = self.v * self.s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = self.v - c;
        let (r, g, b) = match h as u8 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let byte = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
        Rgba {
            r: byte(r + m),
            g: byte(g + m),
            b: byte(b + m),
            a: byte(self.a),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hex_boundary_and_copy_match_main() {
        assert_eq!(Rgba::parse(" aBcDeF80 ").unwrap().hex(), "#abcdef80");
        for text in [
            "#fff",
            "#1234",
            "transparent",
            "#12zz5678",
            "#123456789",
            "#12345é",
        ] {
            assert!(Rgba::parse(text).is_none(), "{text}");
        }
        assert_eq!(
            Rgba {
                r: 1,
                g: 2,
                b: 3,
                a: 128
            }
            .rgb_text(),
            "rgba(1, 2, 3, 0.5)"
        );
        assert_eq!(Rgba::rgb(1, 2, 3).rgb_text(), "rgb(1, 2, 3)");
    }
    #[test]
    fn grayscale_keeps_hue_and_colors_round_trip() {
        let mut hsv = Hsva::new(Rgba::rgb(0, 0, 255));
        hsv.load(Rgba::rgb(127, 127, 127));
        assert_eq!(hsv.h, 240.0);
        for r in [0, 17, 68, 127, 239, 255] {
            for g in [0, 68, 128, 255] {
                for b in [0, 68, 128, 255] {
                    let c = Rgba { r, g, b, a: 87 };
                    let result = Hsva::new(c).color();
                    assert_eq!(result, c);
                }
            }
        }
    }
}
