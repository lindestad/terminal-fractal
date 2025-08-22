// Shade & color utilities (smoothed ramp with more gradual transitions)
pub const SHADES: &[char] = &[' ', '.', ':', '-', '=', '+', '*', 'o', 'O', '#', '█'];

#[inline]
pub fn hsv_to_256(h_deg: f64, s: f64, v: f64) -> u8 {
    let h = (h_deg % 360.0 + 360.0) % 360.0 / 60.0;
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    let (r, g, b) = (r1 + m, g1 + m, b1 + m);
    if s < 0.08 {
        let gray = (v * 23.0).round() as u8;
        return 232 + gray.min(23);
    }
    let ri = (r * 5.0).clamp(0.0, 5.0).round() as u8;
    let gi = (g * 5.0).clamp(0.0, 5.0).round() as u8;
    let bi = (b * 5.0).clamp(0.0, 5.0).round() as u8;
    16 + 36 * ri + 6 * gi + bi
}

#[inline]
pub fn shade(norm: f64) -> char {
    // Slight gamma to bias toward darker chars longer
    let gamma = 0.85;
    let idx = (norm.powf(gamma) * (SHADES.len() - 1) as f64).clamp(0.0, (SHADES.len() - 1) as f64)
        as usize;
    SHADES[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hsv_range() {
        for h in (0..360).step_by(90) {
            let c = hsv_to_256(h as f64, 0.9, 1.0);
            assert!((c..=255).contains(&c));
        }
    }
    #[test]
    fn shade_density_progresses() {
        // Ensure later norm values don't map to an earlier index in the shade ramp
        let mut last_idx = 0usize;
        for i in 0..50 {
            let n = i as f64 / 49.0;
            let ch = shade(n);
            let idx = SHADES.iter().position(|c| *c == ch).unwrap();
            assert!(idx >= last_idx);
            last_idx = idx;
        }
    }
}
