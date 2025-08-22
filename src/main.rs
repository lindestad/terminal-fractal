//! Simplified animated Julia fractal renderer (only 'q' or Ctrl+C to quit)
mod color; // only shared module retained

use color::{hsv_to_256, shade};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use num_complex::Complex64;
use std::{
    io::{self, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

// RAII terminal restore
struct TermGuard;
impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut o = io::stdout();
        let _ = execute!(o, cursor::Show, LeaveAlternateScreen);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Running flag (Ctrl+C)
    let running = Arc::new(AtomicBool::new(true));
    {
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })?;
    }

    // Terminal init
    execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;
    let _g = TermGuard;
    let mut out = io::stdout();

    // Constants
    let max_iters: usize = 120;
    let base_c = Complex64::new(-0.8, 0.156); // base Julia parameter center
    // Smooth wandering (damped random walk) parameters
    let radius = 0.40; // soft bound for |offset|
    let accel_strength = 1.2; // random acceleration magnitude baseline
    let damping = 0.85; // velocity damping (0..1) higher => more damping
    let mut last_time = Instant::now();
    // State for wandering c offset relative to base
    let mut offset = Complex64::new(0.0, 0.0);
    let mut vel = Complex64::new(0.0, 0.0);
    // Tiny PRNG (xorshift64*) to avoid external dependency
    let mut rng: u64 = 0x9e3779b97f4a7c15; // seed
    #[inline]
    fn next_f(r: &mut u64) -> f64 {
        let mut x = *r;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        *r = x;
        let v = x.wrapping_mul(0x2545F4914F6CDD1D);
        // map to [-1,1]
        ((v >> 11) as f64) * (1.0 / ((1u64 << 53) as f64)) * 2.0 - 1.0
    }
    let target_fps = 60.0;
    let target_dt = Duration::from_secs_f64(1.0 / target_fps);
    let mut frame: u64 = 0;
    let start = Instant::now();
    let mut fps_smooth = target_fps;

    while running.load(Ordering::SeqCst) {
        frame += 1;
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f64();
        last_time = now;
        let frame_start = now;

        // Input (only quit)
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => {
                    if (modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c'))
                        || code == KeyCode::Char('q')
                    {
                        running.store(false, Ordering::SeqCst);
                        break;
                    }
                }
                _ => {}
            }
        }

        let (tw, th) = terminal::size().unwrap_or((80, 24));
        let width = tw as usize;
        let height = th.saturating_sub(1) as usize; // leave last line for HUD

        // Advance wandering animation
        let dt_c = dt.min(0.1); // clamp large pauses
        let ax = next_f(&mut rng) * accel_strength;
        let ay = next_f(&mut rng) * accel_strength;
        let acc = Complex64::new(ax, ay);
        // Damped velocity + random acceleration
        vel = vel * (1.0 - damping * dt_c) + acc * dt_c;
        offset += vel * dt_c;
        // Soft boundary: if outside radius, pull inward (spring-like)
        let rlen = offset.norm();
        if rlen > radius {
            let pull = (rlen - radius) / rlen;
            offset -= offset * pull * 0.6; // pull fraction inward
            // also damp velocity outward component
            vel -= offset * (vel.re * offset.re + vel.im * offset.im) / (offset.norm_sqr() + 1e-12)
                * 0.5;
        }
        // Slightly limit extreme velocity
        if vel.norm() > radius * 2.0 {
            vel *= 0.5;
        }
        let c = base_c + offset;

        // Render Julia (no diffing; redraw whole frame)
        queue!(out, cursor::MoveTo(0, 0))?;
        for y in 0..height {
            let im = (y as f64 / height as f64) * 2.0 - 1.0; // [-1,1]
            let mut prev_color: Option<u8> = None;
            for x in 0..width {
                let re = (x as f64 / width as f64) * 3.0 - 1.5; // [-1.5,1.5]
                let mut z = Complex64::new(re, im);
                let mut iters = 0usize;
                while z.norm_sqr() <= 4.0 && iters < max_iters {
                    z = z * z + c;
                    iters += 1;
                }
                if iters >= max_iters {
                    if prev_color.is_some() {
                        write!(out, "\x1b[0m")?;
                        prev_color = None;
                    }
                    out.write_all(b" ")?;
                } else {
                    let norm = iters as f64 / max_iters as f64;
                    let color = hsv_to_256(norm * 360.0, 0.9, 1.0);
                    if prev_color != Some(color) {
                        write!(out, "\x1b[38;5;{}m", color)?;
                        prev_color = Some(color);
                    }
                    let ch = shade(norm);
                    write!(out, "{ch}")?;
                }
            }
            if prev_color.is_some() {
                write!(out, "\x1b[0m")?;
            }
            out.write_all(b"\n")?;
        }

        // HUD
        let frame_time = frame_start.elapsed().as_secs_f64();
        let fps_inst = if frame_time > 0.0 {
            1.0 / frame_time
        } else {
            target_fps
        };
        fps_smooth = fps_smooth * 0.85 + fps_inst * 0.15;
        queue!(out, cursor::MoveTo(0, th.saturating_sub(1)))?;
        queue!(out, terminal::Clear(ClearType::CurrentLine))?;
        write!(
            out,
            "Julia anim | c=({:+.3},{:+.3}) | Frame {} | FPS {:.1} (q/Ctrl+C to quit)",
            c.re, c.im, frame, fps_smooth
        )?;
        out.flush()?;

        // Frame pacing
        let used = frame_start.elapsed();
        if used < target_dt {
            std::thread::sleep(target_dt - used);
        }
    }

    let total = start.elapsed().as_secs_f64();
    let avg = if total > 0.0 {
        frame as f64 / total
    } else {
        0.0
    };
    println!("Exited. Frames: {frame} Time: {total:.2}s Avg FPS: {avg:.2}");
    Ok(())
}
