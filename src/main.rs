use std::time::SystemTime;

use image::{Rgb, RgbImage};

const WIDTH: u32 = 1920 * 2;
const HEIGHT: u32 = 1080 * 2;

const TOTAL_UNITS_WIDE: f64 = 4.0;

const MAX_ITER: u32 = 512;
const CX: f64 = -0.981;
const CY: f64 = -0.277;

// const COL_R: u8 = 4;
// const COL_G: u8 = 8;
// const COL_B: u8 = 12;
const COL_R: u8 = 12;
const COL_G: u8 = 5;
const COL_B: u8 = 10;

const WIDTH_F: f64 = WIDTH as f64;
const HEIGHT_F: f64 = HEIGHT as f64;
const PIXELS_PER_UNIT: f64 = WIDTH_F / TOTAL_UNITS_WIDE;

fn main() {
	let start_time = SystemTime::now();

	let mut img = RgbImage::new(WIDTH, HEIGHT);
	for y in 0..HEIGHT {
		for x in 0..WIDTH {
			let pixel = fractal(x as f64, y as f64);
			img.put_pixel(x, y, pixel);
		}
	}
	println!(
		"Generating took {} ms",
		start_time.elapsed().unwrap().as_millis()
	);

	let filename = format!("julia_set_cx{}_cy{}.png", CX, CY);
	img.save(filename).unwrap();
}

fn fractal(x: f64, y: f64) -> Rgb<u8> {
	let mut x = (x - WIDTH_F / 2.0) / PIXELS_PER_UNIT;
	let mut y = (y - HEIGHT_F / 2.0) / PIXELS_PER_UNIT;

	let mut iterations = 0;

	while (x * x + y * y) < 4.0 {
		(x, y) = (
			x * x - y * y + CX, //
			2.0 * x * y + CY,
		);

		iterations += 1;
		if iterations == MAX_ITER {
			return Rgb([0, 0, 0]);
		}
	}

	let i = iterations.min(255) as u8;
	Rgb([
		i.saturating_mul(COL_R),
		i.saturating_mul(COL_G),
		i.saturating_mul(COL_B),
	])
}
