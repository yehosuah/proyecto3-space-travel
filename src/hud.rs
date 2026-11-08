//! Letras geométricas de 5x7 dibujadas directamente en el framebuffer.
use crate::framebuffer::Framebuffer;

pub fn panel(f: &mut Framebuffer, x: usize, y: usize, w: usize, h: usize, alpha: f32) {
    for py in y..(y + h).min(f.height) {
        for px in x..(x + w).min(f.width) {
            f.blend(px as i32, py as i32, 0x07121c, alpha);
        }
    }
}
pub fn circle(f: &mut Framebuffer, x: f32, y: f32, r: f32, color: u32) {
    for py in (y - r - 1.0) as i32..=(y + r + 1.0) as i32 {
        for px in (x - r - 1.0) as i32..=(x + r + 1.0) as i32 {
            let d = ((px as f32 - x).powi(2) + (py as f32 - y).powi(2)).sqrt();
            f.blend(px, py, color, (r + 0.5 - d).clamp(0.0, 1.0));
        }
    }
}

pub fn rect(f: &mut Framebuffer, x: usize, y: usize, w: usize, h: usize, color: u32) {
    for j in y..(y + h).min(f.height) {
        for i in x..(x + w).min(f.width) {
            f.pixels[j * f.width + i] = color;
        }
    }
}

pub fn text(f: &mut Framebuffer, text: &str, x: usize, y: usize, scale: usize, color: u32) {
    for (index, ch) in text.chars().enumerate() {
        if matches!(ch, 'Á' | 'É' | 'Í' | 'Ó' | 'Ú') {
            rect(
                f,
                x + (index * 6 + 3) * scale,
                y.saturating_sub(3 * scale),
                scale,
                scale,
                color,
            );
            rect(
                f,
                x + (index * 6 + 2) * scale,
                y.saturating_sub(2 * scale),
                scale,
                scale,
                color,
            );
        }
        for (row, bits) in glyph(ch).iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    rect(
                        f,
                        x + (index * 6 + col) * scale,
                        y + row * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
    }
}

fn glyph(ch: char) -> [u8; 7] {
    let ch = match ch {
        'Á' => 'A',
        'É' => 'E',
        'Í' => 'I',
        'Ó' => 'O',
        'Ú' => 'U',
        _ => ch,
    };
    match ch {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [31, 4, 4, 4, 4, 4, 31],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '.' => [0, 0, 0, 0, 0, 4, 4],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        '[' => [14, 8, 8, 8, 8, 8, 14],
        ']' => [14, 2, 2, 2, 2, 2, 14],
        '=' => [0, 0, 31, 0, 31, 0, 0],
        _ => [0; 7],
    }
}
