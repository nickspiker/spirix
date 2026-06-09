//! Font ROM and framebuffer bitmap generator for FPGA NTSC display.
//!
//! Modes: gen_font_rom font <font.ttf> <output.mem> [size] 8×8 character ROM (128 chars × 8 bytes = 1024 bytes)
//!
//!   gen_font_rom bitmap <font.ttf> <output.mem> <width> <height> <text> Full framebuffer bitmap, text centered, 1 bit/pixel packed into bytes. Output: (width/8 * height) lines of 2-hex-digit values, row-major, MSB = leftmost pixel.

use fontdue::{Font, FontSettings};
use spirix::ScalarF4E4;
use std::env;
use std::fs;
use std::io::Write;

const THRESHOLD: u8 = 80;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  gen_font_rom font  <font.ttf> <output.mem> [size]");
        eprintln!("  gen_font_rom bitmap <font.ttf> <output.mem> <width> <height> <text>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "font" => gen_font(&args[2..]),
        "bitmap" => gen_bitmap(&args[2..]),
        "oled-grid" => gen_oled_grid(&args[2..]),
        "tiles" => gen_tiles(&args[2..]),
        _ => {
            eprintln!("Unknown mode: {}. Use 'font', 'bitmap', 'oled-grid', or 'tiles'.", args[1]);
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Mode: tiles — per-glyph 8bpp greyscale bitmaps, one byte per .mem line. Each glyph is tile_w × tile_h bytes, raster-scan order (row-major, top-left first). Glyph N is at offset N × (tile_w × tile_h) bytes. Glyphs are rendered with the natural font aspect (taller than wide for digits) and centered within the tile.
// ============================================================================
fn gen_tiles(args: &[String]) {
    if args.len() < 5 {
        eprintln!("Usage: gen_font_rom tiles <font.ttf> <output.mem> <chars> <tile_w> <tile_h>");
        eprintln!("  Example: gen_font_rom tiles font.ttf glyphs.mem \"0123456789.\" 16 16");
        std::process::exit(1);
    }
    let font = load_font(&args[0]);
    let out_path = &args[1];
    let chars = &args[2];
    let tile_w: usize = args[3].parse().expect("Invalid tile_w");
    let tile_h: usize = args[4].parse().expect("Invalid tile_h");

    // Binary search for the largest font size that fits any of the requested glyphs within tile_w (with a small horizontal margin) and tile_h. Use the tile height as the upper bound of font size since we want the glyph cap height to consume most of the cell.
    let mut lo: f64 = 1.0;
    let mut hi: f64 = (tile_h as f64) * 1.5;
    for _ in 0..32 {
        let mid = (lo + hi) / 2.0;
        let px = ScalarF4E4::from(mid);
        let mut max_w = 0i32;
        let mut max_ascent = 0i32;
        let mut max_descent = 0i32;
        for ch in chars.chars() {
            let m = font.metrics(ch, px);
            if (m.advance_width.ceil().to_isize() as i32) > max_w {
                max_w = m.advance_width.ceil().to_isize() as i32;
            }
            let top = m.ymin + m.height as i32;
            if top > max_ascent { max_ascent = top; }
            if m.ymin < max_descent { max_descent = m.ymin; }
        }
        let total_h = max_ascent - max_descent;
        // Allow content to fill the full tile width (no padding).
        if max_w <= tile_w as i32 && total_h <= tile_h as i32 - 1 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let px = ScalarF4E4::from(lo);

    let mut out = fs::File::create(out_path).expect("Failed to create output");
    let mut total_bytes = 0usize;
    let mut max_ascent_global = 0i32;
    let mut max_descent_global = 0i32;
    for ch in chars.chars() {
        let m = font.metrics(ch, px);
        let top = m.ymin + m.height as i32;
        if top > max_ascent_global { max_ascent_global = top; }
        if m.ymin < max_descent_global { max_descent_global = m.ymin; }
    }
    let baseline = (tile_h as i32 + (max_ascent_global - max_descent_global)) / 2 - max_descent_global;

    for ch in chars.chars() {
        let (m, bitmap) = font.rasterize(ch, px);
        let mut tile = vec![0u8; tile_w * tile_h];
        let cx = (tile_w as i32 - m.advance_width.ceil().to_isize() as i32) / 2;
        for gy in 0..m.height as i32 {
            let py = baseline - max_ascent_global + gy + (max_ascent_global - m.ymin - m.height as i32);
            if py < 0 || py >= tile_h as i32 { continue; }
            for gx in 0..m.width as i32 {
                let tx = cx + m.xmin + gx;
                if tx < 0 || tx >= tile_w as i32 { continue; }
                let v = bitmap[(gy * m.width as i32 + gx) as usize];
                tile[py as usize * tile_w + tx as usize] = v;
            }
        }
        for byte in &tile {
            writeln!(out, "{:02x}", byte).unwrap();
        }
        total_bytes += tile.len();
    }

    eprintln!("Generated {}: {} bytes, {} glyphs of {}×{} ({:.1}px font)",
              out_path, total_bytes, chars.chars().count(), tile_w, tile_h, lo);

    // ASCII preview of the first few glyphs
    eprintln!("Preview (first {} glyphs):", chars.chars().count().min(11));
    let mut preview_chars: Vec<char> = chars.chars().take(11).collect();
    let _ = preview_chars.len();
    for ch in &preview_chars {
        eprintln!("  '{}':", ch);
        let (m, bitmap) = font.rasterize(*ch, px);
        let mut tile = vec![0u8; tile_w * tile_h];
        let cx = (tile_w as i32 - m.advance_width.ceil().to_isize() as i32) / 2;
        for gy in 0..m.height as i32 {
            let py = baseline - max_ascent_global + gy + (max_ascent_global - m.ymin - m.height as i32);
            if py < 0 || py >= tile_h as i32 { continue; }
            for gx in 0..m.width as i32 {
                let tx = cx + m.xmin + gx;
                if tx < 0 || tx >= tile_w as i32 { continue; }
                tile[py as usize * tile_w + tx as usize] = bitmap[(gy * m.width as i32 + gx) as usize];
            }
        }
        for y in 0..tile_h {
            let mut line = String::from("    ");
            for x in 0..tile_w {
                line.push(if tile[y * tile_w + x] > THRESHOLD { '#' } else { '.' });
            }
            eprintln!("{}", line);
        }
    }
}

fn load_font(path: &str) -> Font {
    let bytes = fs::read(path).expect("Failed to read font file");
    Font::from_bytes(bytes, FontSettings::default()).expect("Failed to parse font")
}

// ============================================================================
// Mode: bitmap — render text into a full framebuffer ============================================================================
fn gen_bitmap(args: &[String]) {
    if args.len() < 5 {
        eprintln!("Usage: gen_font_rom bitmap <font.ttf> <output.mem> <width> <height> <text> [--flip] [--par <ratio>]");
        std::process::exit(1);
    }

    let font = load_font(&args[0]);
    let out_path = &args[1];
    let width: usize = args[2].parse().expect("Invalid width");
    let height: usize = args[3].parse().expect("Invalid height");
    let text = &args[4];

    // Parse optional flags
    let mut flip = false;
    let mut par: f64 = 1.0; // pixel aspect ratio (width/height of one pixel)
    let mut i = 5;
    while i < args.len() {
        match args[i].as_str() {
            "--flip" => { flip = true; i += 1; }
            "--par" => { par = args[i+1].parse().expect("Invalid PAR"); i += 2; }
            _ => { i += 1; }
        }
    }

    assert!(width % 8 == 0, "Width must be a multiple of 8");

    // Effective width in square-pixel space for text layout
    let eff_width = (width as f64 * par) as usize;

    // Binary search for the largest font size that fits in effective space
    let mut lo: f64 = 1.0;
    let mut hi: f64 = height as f64 * 2.0;
    for _ in 0..32 {
        let mid = (lo + hi) / 2.0;
        let px = ScalarF4E4::from(mid);
        let tw: isize = text.chars()
            .map(|ch| font.metrics(ch, px).advance_width.ceil().to_isize())
            .sum();
        let th = mid;
        if tw <= eff_width as isize && th <= height as f64 {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    let px = ScalarF4E4::from(lo);

    // Measure total width and find baseline
    let mut total_width: isize = 0;
    let mut max_ascent: i32 = 0;
    let mut max_descent: i32 = 0;
    for ch in text.chars() {
        let (metrics, _) = font.rasterize(ch, px);
        let top = metrics.ymin + metrics.height as i32;
        if top > max_ascent { max_ascent = top; }
        if metrics.ymin < max_descent { max_descent = metrics.ymin; }
        total_width += metrics.advance_width.ceil().to_isize();
    }

    let text_height = (max_ascent - max_descent) as usize;

    // Render into square-pixel buffer at effective width
    let mut sq_pixels = vec![false; eff_width * height];
    let x_start = ((eff_width as isize - total_width) / 2).max(0) as usize;
    let y_baseline = (height + text_height) / 2 - max_ascent as usize;
    let mut cursor_x = x_start as i32;

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, px);
        let gw = metrics.width as i32;
        let gh = metrics.height as i32;

        for gy in 0..gh {
            let py = y_baseline as i32 + (max_ascent - metrics.ymin - gh + gy);
            if py < 0 || py >= height as i32 { continue; }
            for gx in 0..gw {
                let px_x = cursor_x + metrics.xmin + gx;
                if px_x < 0 || px_x >= eff_width as i32 { continue; }
                let coverage = bitmap[(gy * gw + gx) as usize];
                if coverage > THRESHOLD {
                    sq_pixels[py as usize * eff_width + px_x as usize] = true;
                }
            }
        }
        cursor_x += metrics.advance_width.ceil().to_isize() as i32;
    }

    // Stretch to output width (nearest-neighbor) and optionally flip 180°
    let mut pixels = vec![false; width * height];
    for y in 0..height {
        for x in 0..width {
            let sx = (x as f64 * par) as usize;
            let sx = sx.min(eff_width - 1);
            if sq_pixels[y * eff_width + sx] {
                let (ox, oy) = if flip {
                    (width - 1 - x, height - 1 - y)
                } else {
                    (x, y)
                };
                pixels[oy * width + ox] = true;
            }
        }
    }

    // Write as hex bytes (8 pixels per byte, MSB = leftmost)
    let bytes_per_row = width / 8;
    let mut out = fs::File::create(out_path).expect("Failed to create output");
    let mut total_set = 0usize;

    for y in 0..height {
        for bx in 0..bytes_per_row {
            let mut byte: u8 = 0;
            for bit in 0..8 {
                let x = bx * 8 + bit;
                if pixels[y * width + x] {
                    byte |= 1 << (7 - bit);
                    total_set += 1;
                }
            }
            writeln!(out, "{:02x}", byte).unwrap();
        }
    }

    let total_bytes = bytes_per_row * height;
    eprintln!(
        "Generated {}: {}×{} = {} bytes, {} pixels set, font size {:.1}px",
        out_path, width, height, total_bytes, total_set, lo
    );

    // ASCII preview (every Nth row to fit terminal)
    let step = (height / 20).max(1);
    for y in (0..height).step_by(step) {
        let mut line = String::new();
        let col_step = (width / 80).max(1);
        for x in (0..width).step_by(col_step) {
            line.push(if pixels[y * width + x] { '#' } else { '.' });
        }
        eprintln!("{}", line);
    }
}

// ============================================================================
// Mode: oled-grid — 4×4 labeled pass/fail overlay for SH1106 128×64 OLED ============================================================================ Output: 1024 bytes in OLED page format (8 pages × 128 cols). Each byte: bit 0 = top row of page, bit 7 = bottom row. XOR'd with pass/fail bars in hardware for always-visible labels.
fn gen_oled_grid(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: gen_font_rom oled-grid <font.ttf> <output.mem>");
        std::process::exit(1);
    }

    let font = load_font(&args[0]);
    let out_path = &args[1];

    let width = 128usize;
    let height = 64usize;
    let cell_w = 32usize;
    let cell_h = 16usize;

    // Grid labels: F{6-gc}E{3+gr} for grid row gr, grid col gc Columns: frac=64(left) to frac=8(right), Rows: exp=8(top) to exp=64(bottom)
    let labels: [[&str; 4]; 4] = [
        ["F6E3", "F5E3", "F4E3", "F3E3"],
        ["F6E4", "F5E4", "F4E4", "F3E4"],
        ["F6E5", "F5E5", "F4E5", "F3E5"],
        ["F6E6", "F5E6", "F4E6", "F3E6"],
    ];

    // Binary search for largest font size that fits in a cell (4 chars in 32px)
    let mut lo: f64 = 1.0;
    let mut hi: f64 = cell_h as f64;
    for _ in 0..32 {
        let mid = (lo + hi) / 2.0;
        let px = ScalarF4E4::from(mid);
        let tw: isize = "F6E3".chars()
            .map(|ch| font.metrics(ch, px).advance_width.ceil().to_isize())
            .sum();
        if tw <= cell_w as isize && mid <= cell_h as f64 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let px = ScalarF4E4::from(lo);

    // Render each label centered in its cell
    let mut pixels = vec![false; width * height];

    for gr in 0..4 {
        for gc in 0..4 {
            let text = labels[gr][gc];
            let x0 = gc * cell_w;
            let y0 = gr * cell_h;

            // Measure text width and ascent/descent
            let mut tw: i32 = 0;
            let mut max_ascent: i32 = 0;
            let mut max_descent: i32 = 0;
            for ch in text.chars() {
                let (m, _) = font.rasterize(ch, px);
                let top = m.ymin + m.height as i32;
                if top > max_ascent { max_ascent = top; }
                if m.ymin < max_descent { max_descent = m.ymin; }
                tw += m.advance_width.ceil().to_isize() as i32;
            }

            let th = (max_ascent - max_descent) as usize;
            let cx = x0 + (cell_w as i32 - tw) as usize / 2;
            let cy = y0 + (cell_h + th) / 2 - max_ascent as usize;

            let mut cursor_x = cx as i32;
            for ch in text.chars() {
                let (m, bitmap) = font.rasterize(ch, px);
                for gy in 0..m.height as i32 {
                    let py = cy as i32 + (max_ascent - m.ymin - m.height as i32 + gy);
                    if py < 0 || py >= height as i32 { continue; }
                    for gx in 0..m.width as i32 {
                        let px_x = cursor_x + m.xmin + gx;
                        if px_x < 0 || px_x >= width as i32 { continue; }
                        if bitmap[(gy * m.width as i32 + gx) as usize] > THRESHOLD {
                            pixels[py as usize * width + px_x as usize] = true;
                        }
                    }
                }
                cursor_x += m.advance_width.ceil().to_isize() as i32;
            }
        }
    }

    // Convert to OLED page format: 8 pages × 128 cols, bit 0 = top row of page
    let mut out = fs::File::create(out_path).expect("Failed to create output");
    for page in 0..8 {
        for col in 0..128 {
            let mut byte: u8 = 0;
            for bit in 0..8 {
                let y = page * 8 + bit;
                if pixels[y * width + col] {
                    byte |= 1 << bit;
                }
            }
            writeln!(out, "{:02x}", byte).unwrap();
        }
    }

    eprintln!("Generated {}: 1024 bytes (8 pages × 128 cols), font size {:.1}px", out_path, lo);

    // ASCII preview
    let step = (height / 16).max(1);
    for y in (0..height).step_by(step) {
        let mut line = String::new();
        for x in 0..width {
            line.push(if pixels[y * width + x] { '#' } else { '.' });
        }
        eprintln!("{}", line);
    }
}

// ============================================================================
// Mode: font — 8×8 character ROM ============================================================================
fn gen_font(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: gen_font_rom font <font.ttf> <output.mem> [size]");
        std::process::exit(1);
    }

    let font = load_font(&args[0]);
    let out_path = &args[1];
    let px_size: f64 = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(9.0);
    let px = ScalarF4E4::from(px_size);
    let mut out = fs::File::create(out_path).expect("Failed to create output");

    let cell_w = 8usize;
    let cell_h = 8usize;
    let num_chars = 128usize;

    for code in 0..num_chars {
        if code < 32 || code == 127 {
            for _ in 0..cell_h {
                writeln!(out, "00").unwrap();
            }
            continue;
        }

        let ch = code as u8 as char;
        let (metrics, bitmap) = font.rasterize(ch, px);
        let gw = metrics.width;
        let gh = metrics.height;
        let x_off = ((cell_w as i32 - gw as i32) / 2).max(0) as usize;
        let y_off = ((cell_h as i32 - gh as i32) / 2).max(0) as usize;
        let gx_start = if gw > cell_w { (gw - cell_w) / 2 } else { 0 };
        let gy_start = if gh > cell_h { (gh - cell_h) / 2 } else { 0 };

        for row in 0..cell_h {
            let mut byte: u8 = 0;
            for col in 0..cell_w {
                let gx = col as i32 - x_off as i32 + gx_start as i32;
                let gy = row as i32 - y_off as i32 + gy_start as i32;
                let coverage = if gx >= 0 && gx < gw as i32 && gy >= 0 && gy < gh as i32 {
                    bitmap[gy as usize * gw + gx as usize]
                } else {
                    0
                };
                if coverage > THRESHOLD {
                    byte |= 1 << (7 - col);
                }
            }
            writeln!(out, "{:02x}", byte).unwrap();
        }
    }

    eprintln!("Generated {}: {} bytes ({} chars × {} rows)", out_path, num_chars * cell_h, num_chars, cell_h);
}
