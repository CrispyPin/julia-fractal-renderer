use eframe::epaint::Vec2;
use image::{Rgb, RgbImage};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RenderOptions {
	pub width: u32,
	pub height: u32,
	pub unit_width: f64,
	pub max_iterations: u32,
	pub cx: f64,
	pub cy: f64,
	pub fill_style: FillStyle,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum FillStyle {
	Bright,
	Black,
}

impl Default for RenderOptions {
	fn default() -> Self {
		Self {
			width: 512,
			height: 512,
			unit_width: 4.0,
			max_iterations: 128,
			cx: -0.8,
			cy: -0.27,
			fill_style: FillStyle::Bright,
		}
	}
}

pub fn view_point(q: &RenderOptions, image: RgbImage) -> RgbImage {
	apply_fn(image, q, |x, y| {
		let len = (Vec2::new(x as f32, y as f32) - Vec2::new(q.cx as f32, q.cy as f32)).length();
		if len < 0.04 {
			Some(Rgb([0, 255, 255]))
		} else if len < 0.05 {
			Some(Rgb([255; 3]))
		} else {
			None
		}
	})
}

pub fn render(q: &RenderOptions, color: (u8, u8, u8)) -> RgbImage {
	let img = RgbImage::new(q.width, q.height);
	apply_fn(img, q, |x, y| {
		let i = julia(x, y, q.cx, q.cy, q.max_iterations);
		if q.fill_style == FillStyle::Black && i == q.max_iterations {
			None
		} else {
			let i = i.min(255) as u8;
			Some(Rgb([
				i.saturating_mul(color.0),
				i.saturating_mul(color.1),
				i.saturating_mul(color.2),
			]))
		}
	})
}

fn apply_fn<F>(mut image: RgbImage, q: &RenderOptions, op: F) -> RgbImage
where
	F: Fn(f64, f64) -> Option<Rgb<u8>>,
{
	let width = q.width as f64;
	let height = q.height as f64;
	let ppu = width / q.unit_width;

	for y in 0..q.height {
		for x in 0..q.width {
			let sx = (x as f64 - width / 2.0) / ppu;
			let sy = (y as f64 - height / 2.0) / ppu;
			if let Some(pixel) = op(sx, sy) {
				image.put_pixel(x, y, pixel);
			}
		}
	}
	image
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
