use num_complex::Complex64;

fn main() {
    let width: usize = 80;
    let height: usize = 40;
    let c = Complex64::new(-0.8, 0.156);

    for y in 0..height {
        for x in 0..width {
            let re = (x as f64 / width as f64) * 3.0 - 1.5;
            let im = (y as f64 / height as f64) * 2.0 - 1.0;
            let mut z = Complex64::new(re, im);
            let mut iters: usize = 0;
            let max_iters: usize = 80;
            while z.norm_sqr() <= 4.0 && iters < max_iters {
                z = z * z + c;
                iters += 1;
            }
            let ch = match iters {
                0..=8 => ' ',
                9..=16 => '.',
                17..=32 => '*',
                33..=64 => 'o',
                _ => '#',
            };
            print!("{ch}");
        }
        println!();
    }
}
