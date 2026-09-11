//! Small native drawing helpers. GDI+ and logo resources are owned by the window.
use std::ptr::null_mut;
use windows_sys::Win32::{
    Foundation::{HANDLE, RECT},
    Graphics::{Gdi::*, GdiPlus::*},
    UI::{Shell::SHCreateMemStream, WindowsAndMessaging::*},
};

pub struct Visuals {
    token: usize,
    pub fonts: [HANDLE; 2],
    pub app: HICON,
    pub app_small: HICON,
    pub mascot: *mut GpBitmap,
    mascot_stream: *mut std::ffi::c_void,
    pub codex: HICON,
    pub claude: HICON,
    pub orca: HICON,
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
            let logo = include_bytes!("../assets/waid.png");
            let data = include_bytes!("../../assets/waid-mascot.png");
            let mascot_stream = SHCreateMemStream(data.as_ptr(), data.len() as u32);
            let mut mascot = null_mut();
            if !mascot_stream.is_null() {
                GdipCreateBitmapFromStream(mascot_stream, &mut mascot);
            }
            Self {
                token,
                fonts: [
                    include_bytes!("../assets/fonts/Pretendard-Bold.otf").as_slice(),
                    include_bytes!("../assets/fonts/Pretendard-SemiBold.otf").as_slice(),
                ]
                .map(|bytes| {
                    let mut count = 0;
                    AddFontMemResourceEx(
                        bytes.as_ptr() as *const _,
                        bytes.len() as u32,
                        null_mut(),
                        &mut count,
                    )
                }),
                app: png_icon_sized(logo, GetSystemMetrics(SM_CXICON)),
                app_small: png_icon_sized(logo, GetSystemMetrics(SM_CXSMICON)),
                mascot,
                mascot_stream,
                codex: png_icon(include_bytes!("../assets/openai.png")),
                claude: ico_icon(include_bytes!("../assets/claude.ico")),
                orca: png_icon(include_bytes!("../assets/orca.png")),
            }
        }
    }
    pub unsafe fn draw_mascot(&self, dc: HDC, x: i32, y: i32, size: i32) -> bool {
        let mut graphics = null_mut();
        if self.mascot.is_null() || GdipCreateFromHDC(dc, &mut graphics) != 0 {
            return false;
        }
        GdipSetInterpolationMode(graphics, InterpolationModeHighQualityBicubic);
        GdipSetPixelOffsetMode(graphics, PixelOffsetModeHalf);
        let drawn = GdipDrawImageRectI(graphics, self.mascot.cast(), x, y, size, size) == 0;
        GdipDeleteGraphics(graphics);
        drawn
    }
    pub unsafe fn logo(&self, dc: HDC, agent_id: &str, x: i32, y: i32, size: i32) -> bool {
        let icon = match agent_id {
            "codex" => self.codex,
            "claude" => self.claude,
            "orca" => self.orca,
            _ => null_mut(),
        };
        !icon.is_null() && DrawIconEx(dc, x, y, icon, size, size, 0, null_mut(), DI_NORMAL) != 0
    }
}
impl Drop for Visuals {
    fn drop(&mut self) {
        unsafe {
            for font in self.fonts {
                if !font.is_null() {
                    RemoveFontMemResourceEx(font);
                }
            }
            for icon in [
                self.app,
                self.app_small,
                self.codex,
                self.claude,
                self.orca,
            ] {
                if !icon.is_null() {
                    DestroyIcon(icon);
                }
            }
            if !self.mascot.is_null() {
                GdipDisposeImage(self.mascot.cast());
            }
            // GDI+ needs its source stream until the image is disposed.
            if !self.mascot_stream.is_null() {
                let vtable = *(self.mascot_stream as *const *const windows_sys::core::IUnknown_Vtbl);
                ((*vtable).Release)(self.mascot_stream);
            }
            if self.token != 0 {
                GdiplusShutdown(self.token);
            }
        }
    }
}
unsafe fn png_icon(data: &[u8]) -> HICON {
    png_icon_sized(data, 96)
}
unsafe fn png_icon_sized(data: &[u8], size: i32) -> HICON {
    let mut bytes = data.to_vec();
    CreateIconFromResourceEx(
        bytes.as_mut_ptr(),
        bytes.len() as u32,
        1,
        0x00030000,
        size.max(16),
        size.max(16),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mascot_preserves_transparency_and_shape_at_common_dpis() {
        let visuals = Visuals::new();
        unsafe {
            for size in [32, 40, 48, 64, 96] {
                let dc = CreateCompatibleDC(null_mut());
                assert!(!dc.is_null());
                let info = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: size, biHeight: -size, biPlanes: 1, biBitCount: 32,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let mut bits = null_mut();
                let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
                assert!(!bitmap.is_null());
                let old = SelectObject(dc, bitmap);
                let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, (size * size) as usize);
                for background in [0xfff5f7fb, 0xff102038] {
                    pixels.fill(background);
                    assert!(visuals.draw_mascot(dc, 0, 0, size));
                    GdiFlush();
                    assert_eq!(pixels[0] & 0xffffff, background & 0xffffff, "transparent corner at {size}px");
                    let center = pixels[(size * (size / 2) + size / 2) as usize];
                    assert!(center & 255 > (center >> 16) & 255, "blue body at {size}px");
                    assert!(pixels.iter().filter(|p| **p & 0xffffff != background & 0xffffff).count() > pixels.len() / 3);
                    // Keep the actual native rendering available for visual inspection.
                    let mut bmp = b"BM".to_vec();
                    bmp.extend_from_slice(&(54 + pixels.len() as u32 * 4).to_le_bytes());
                    bmp.extend_from_slice(&[0; 4]);
                    bmp.extend_from_slice(&54u32.to_le_bytes());
                    bmp.extend_from_slice(std::slice::from_raw_parts(&info.bmiHeader as *const _ as *const u8, 40));
                    bmp.extend(pixels.iter().flat_map(|p| p.to_le_bytes()));
                    let path = std::env::temp_dir().join(format!("waid-mascot-{size}-{background:x}.bmp"));
                    std::fs::write(path, bmp).unwrap();
                }
                SelectObject(dc, old);
                DeleteObject(bitmap);
                DeleteDC(dc);
            }
        }
    }
}
