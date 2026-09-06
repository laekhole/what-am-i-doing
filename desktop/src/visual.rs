//! Small native drawing helpers. GDI+ and logo resources are owned by the window.
use std::ptr::null_mut;
use windows_sys::Win32::{
    Foundation::RECT,
    Graphics::{Gdi::*, GdiPlus::*},
    UI::WindowsAndMessaging::*,
};

pub struct Visuals {
    token: usize,
    pub codex: HICON,
    pub claude: HICON,
}
impl Visuals {
    pub fn new() -> Self {
        unsafe {
            let mut token = 0;
            let input = GdiplusStartupInput {
                GdiplusVersion: 1,
                ..Default::default()
            };
            GdiplusStartup(&mut token, &input, null_mut());
            Self {
                token,
                codex: png_icon(include_bytes!("../assets/openai.png")),
                claude: ico_icon(include_bytes!("../assets/claude.ico")),
            }
        }
    }
    pub unsafe fn logo(&self, dc: HDC, agent_id: &str, x: i32, y: i32, size: i32) -> bool {
        let icon = match agent_id {
            "codex" => self.codex,
            "claude" => self.claude,
            _ => null_mut(),
        };
        !icon.is_null() && DrawIconEx(dc, x, y, icon, size, size, 0, null_mut(), DI_NORMAL) != 0
    }
}
impl Drop for Visuals {
    fn drop(&mut self) {
        unsafe {
            for icon in [self.codex, self.claude] {
                if !icon.is_null() {
                    DestroyIcon(icon);
                }
            }
            if self.token != 0 {
                GdiplusShutdown(self.token);
            }
        }
    }
}
unsafe fn png_icon(data: &[u8]) -> HICON {
    let mut bytes = data.to_vec();
    CreateIconFromResourceEx(
        bytes.as_mut_ptr(),
        bytes.len() as u32,
        1,
        0x00030000,
        96,
        96,
        LR_DEFAULTCOLOR,
    )
}
unsafe fn ico_icon(data: &[u8]) -> HICON {
    let word = |i| u16::from_le_bytes([data[i], data[i + 1]]) as usize;
    let dword = |i| u32::from_le_bytes(data[i..i + 4].try_into().unwrap()) as usize;
    if data.len() < 6 || &data[..4] != [0, 0, 1, 0] {
        return null_mut();
    }
    let mut best = None;
    for i in 0..word(4) {
        let p = 6 + i * 16;
        if p + 16 > data.len() {
            break;
        }
        let size = dword(p + 8);
        let offset = dword(p + 12);
        if offset.checked_add(size).is_none_or(|end| end > data.len()) {
            continue;
        }
        let width = if data[p] == 0 { 256 } else { data[p] as usize };
        if best.is_none_or(|(w, _, _)| width > w) {
            best = Some((width, offset, size));
        }
    }
    match best {
        Some((_, offset, size)) => png_icon(&data[offset..offset + size]),
        None => null_mut(),
    }
}
pub fn blend(a: u32, b: u32, percent: u32) -> u32 {
    [0, 8, 16].into_iter().fold(0, |out, shift| {
        out | (((((a >> shift) & 255) * (100 - percent) + ((b >> shift) & 255) * percent) / 100)
            << shift)
    })
}
fn argb(color: u32) -> u32 {
    0xff000000 | ((color & 255) << 16) | (color & 0xff00) | ((color >> 16) & 255)
}
pub unsafe fn rounded(dc: HDC, rect: RECT, radius: i32, color: u32, border: Option<u32>) {
    let mut graphics = null_mut();
    if GdipCreateFromHDC(dc, &mut graphics) != 0 {
        return;
    }
    GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias);
    let mut path = null_mut();
    if GdipCreatePath(FillModeAlternate, &mut path) == 0 {
        let x = rect.left as f32 + 0.5;
        let y = rect.top as f32 + 0.5;
        let w = (rect.right - rect.left - 1).max(1) as f32;
        let h = (rect.bottom - rect.top - 1).max(1) as f32;
        let d = (radius * 2).max(2) as f32;
        let d = d.min(w).min(h);
        for (ax, ay, angle) in [
            (x, y, 180.),
            (x + w - d, y, 270.),
            (x + w - d, y + h - d, 0.),
            (x, y + h - d, 90.),
        ] {
            GdipAddPathArc(path, ax, ay, d, d, angle, 90.);
        }
        GdipClosePathFigure(path);
        let mut brush = null_mut();
        if GdipCreateSolidFill(argb(color), &mut brush) == 0 {
            GdipFillPath(graphics, brush as _, path);
            GdipDeleteBrush(brush as _);
        }
        if let Some(color) = border {
            let mut pen = null_mut();
            if GdipCreatePen1(argb(color), 1., UnitPixel, &mut pen) == 0 {
                GdipDrawPath(graphics, pen, path);
                GdipDeletePen(pen);
            }
        }
        GdipDeletePath(path);
    }
    GdipDeleteGraphics(graphics);
}
