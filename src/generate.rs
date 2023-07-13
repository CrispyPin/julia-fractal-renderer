use image::{Rgb, RgbImage};

#[derive(Clone)]
pub struct RenderOptions {
	pub width: u32,
	pub height: u32,
	pub unit_width: f64,
	pub max_iterations: u32,
	pub cx: f64,
	pub cy: f64,
	pub fill_style: FillStyle,
}

#[derive(PartialEq, Clone)]
pub enum FillStyle {
	Bright,
	Black,
}

pub fn render(q: &RenderOptions, color: (u8, u8, u8)) -> RgbImage {
	let mut img = RgbImage::new(q.width, q.height);

	let width: f64 = q.width.into();
	let height: f64 = q.height.into();
	let ppu = width / q.unit_width;

	for y in 0..q.height {
		for x in 0..q.width {
			let pixel = {
				let x = (f64::from(x) - width / 2.0) / ppu;
				let y = (f64::from(y) - height / 2.0) / ppu;

				let iter = julia(x, y, q.cx, q.cy, q.max_iterations);
				if q.fill_style == FillStyle::Black && iter == q.max_iterations {
					Rgb([0, 0, 0])
				} else {
					let i = iter.min(255) as u8;
					Rgb([
						i.saturating_mul(color.0),
						i.saturating_mul(color.1),
						i.saturating_mul(color.2),
					])
				}
			};
			img.put_pixel(x, y, pixel);
		}
	}
	img
}

fn julia(mut x: f64, mut y: f64, cx: f64, cy: f64, max_iter: u32) -> u32 {
	let mut iter = 0;
	while (x * x + y * y) < 4.0 && iter < max_iter {
		(x, y) = (
			x * x - y * y + cx, //
			2.0 * x * y + cy,
		);
		iter += 1;
	}
	iter
}
