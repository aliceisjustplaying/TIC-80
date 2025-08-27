use std::fs::File;
use std::io::Write;
use std::path::Path;

#[must_use]
pub fn scale_rgba_nn(src: &[u8], w: u32, h: u32, scale: u32) -> Vec<u8> {
    if scale <= 1 {
        return src.to_vec();
    }
    let w = w as usize;
    let h = h as usize;
    let s = scale as usize;
    let mut out = vec![0u8; w * s * h * s * 4];
    for y in 0..h {
        for x in 0..w {
            let src_idx = (y * w + x) * 4;
            let px = &src[src_idx..src_idx + 4];
            let ox = x * s;
            let oy = y * s;
            for dy in 0..s {
                let row = (oy + dy) * (w * s) * 4;
                for dx in 0..s {
                    let dst = row + (ox + dx) * 4;
                    out[dst..dst + 4].copy_from_slice(px);
                }
            }
        }
    }
    out
}

/// # Errors
/// - Returns an error if the file cannot be created or PNG encoding fails.
pub fn save_png_rgba(path: &Path, w: u32, h: u32, rgba: &[u8]) -> anyhow::Result<()> {
    let file = File::create(path)?;
    write_png_rgba(file, w, h, rgba)?;
    Ok(())
}

/// # Errors
/// - Returns an error if PNG encoding fails.
pub fn encode_png_rgba_to_vec(w: u32, h: u32, rgba: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::new();
    write_png_rgba(&mut buf, w, h, rgba)?;
    Ok(buf)
}

fn write_png_rgba<W: Write>(wtr: W, w: u32, h: u32, rgba: &[u8]) -> anyhow::Result<()> {
    let mut encoder = png::Encoder::new(wtr, w, h);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(rgba)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_nn_2x() {
        // 2x2 checkerboard RGBA
        let w = 2u32;
        let h = 2u32;
        let black = [0, 0, 0, 255];
        let white = [255, 255, 255, 255];
        let mut src = Vec::new();
        src.extend_from_slice(&black);
        src.extend_from_slice(&white);
        src.extend_from_slice(&white);
        src.extend_from_slice(&black);
        let out = scale_rgba_nn(&src, w, h, 2);
        assert_eq!(out.len(), (w * 2 * h * 2 * 4) as usize);
        // Check corners
        let stride = (w as usize) * 2 * 4;
        // top-left
        assert_eq!(&out[0..4], &black);
        // top-right
        assert_eq!(&out[stride - 4..stride], &white);
        // bottom-left
        let last_row = (h as usize) * 2 - 1;
        let idx = last_row * stride;
        assert_eq!(&out[idx..idx + 4], &white);
        // bottom-right
        assert_eq!(&out[idx + stride - 4..idx + stride], &black);
    }

    #[test]
    fn png_roundtrip() {
        let w = 3u32;
        let h = 2u32;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                rgba[i] = u8::try_from((x * 40) % 256).unwrap();
                rgba[i + 1] = u8::try_from((y * 80) % 256).unwrap();
                rgba[i + 2] = 128;
                rgba[i + 3] = 255;
            }
        }
        let bytes = encode_png_rgba_to_vec(w, h, &rgba).unwrap();
        let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0u8; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        assert_eq!(info.width, w);
        assert_eq!(info.height, h);
    }
}
