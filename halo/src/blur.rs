// Separable Gaussian blur of a single-channel image, kernel radius 3 sigma, edges clamped to 0.
pub fn gaussian(a: &mut [f32], w: usize, h: usize, sigma: f32) {
    if sigma <= 0.0 {
        return;
    }
    let r = (sigma * 3.0).ceil() as isize;
    let k: Vec<f32> = (-r..=r).map(|i| (-(i * i) as f32 / (2.0 * sigma * sigma)).exp()).collect();
    let sum: f32 = k.iter().sum();
    let k: Vec<f32> = k.iter().map(|v| v / sum).collect();
    let mut tmp = vec![0.0; a.len()];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for (j, kv) in k.iter().enumerate() {
                let xx = x as isize + j as isize - r;
                if (0..w as isize).contains(&xx) {
                    s += a[y * w + xx as usize] * kv;
                }
            }
            tmp[y * w + x] = s;
        }
    }
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for (j, kv) in k.iter().enumerate() {
                let yy = y as isize + j as isize - r;
                if (0..h as isize).contains(&yy) {
                    s += tmp[yy as usize * w + x] * kv;
                }
            }
            a[y * w + x] = s;
        }
    }
}
