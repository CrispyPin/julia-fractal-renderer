use eframe::epaint::Vec2;
use image::{Rgb, RgbImage};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RenderOptions {
	pub width: usize,
	pub height: usize,
	pub unit_width: f64,
	pub max_iter: u16,
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
			max_iter: 128,
			cx: 0.4,
			cy: -0.2,
			fill_style: FillStyle::Bright,
		}
	}
}

pub fn render_c(q: &RenderOptions, mut image: RgbImage) -> RgbImage {
	let width = q.width as f32;
	let height = q.height as f32;
	let ppu = width / (q.unit_width as f32);

	let target = Vec2::new(q.cx as f32, q.cy as f32);

	for y in 0..q.height {
		for x in 0..q.width {
			let sx = (x as f32 - width / 2.0) / ppu;
			let sy = (y as f32 - height / 2.0) / ppu;

			let len = (Vec2::new(sx, sy) - target).length();
			if len < 0.03 {
				image.put_pixel(x as u32, y as u32, Rgb([0, 120, 120]));
			} else if len < 0.04 {
				image.put_pixel(x as u32, y as u32, Rgb([255; 3]));
			}
		}
	}
	image
}

pub fn color_iteration(iter: u16, color: (u8, u8, u8)) -> Rgb<u8> {
	let i = iter.min(255) as u8;
	Rgb([
		i.saturating_mul(color.0),
		i.saturating_mul(color.1),
		i.saturating_mul(color.2),
	])
}

pub fn render_julia(q: &RenderOptions, color: (u8, u8, u8)) -> RgbImage {
	let mut image = RgbImage::new(q.width as u32, q.height as u32);

	let width = q.width as f64;
	let height = q.height as f64;
	let ppu = width / q.unit_width;

	let fill = match q.fill_style {
		FillStyle::Black => Rgb([0; 3]),
		FillStyle::Bright => color_iteration(q.max_iter, color),
	};

	(0..q.height)
		.into_par_iter()
		.map(|y| {
			let mut row = Vec::with_capacity(q.width);
			for x in 0..q.width {
				let sx = (x as f64 - width / 2.0) / ppu;
				let sy = (y as f64 - height / 2.0) / ppu;
				let i = julia(sx, sy, q.cx, q.cy, q.max_iter);

				if i == q.max_iter {
					row.push(fill);
				} else {
					row.push(color_iteration(i, color));
				}
			}
			row
		})
		.collect::<Vec<_>>()
		.into_iter()
		.enumerate()
		.for_each(|(y, row)| {
			for (x, i) in row.into_iter().enumerate() {
				image.put_pixel(x as u32, y as u32, i);
			}
		});
	image
}

fn julia(mut x: f64, mut y: f64, cx: f64, cy: f64, max_iter: u16) -> u16 {
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
