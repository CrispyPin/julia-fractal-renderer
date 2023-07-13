#![windows_subsystem = "windows"]
use std::time::SystemTime;

use eframe::{
	egui::{self, DragValue, RichText, Slider, TextureOptions},
	epaint::{TextureHandle, Vec2},
	Frame, NativeOptions,
};
use generate::{render, FillStyle, RenderOptions};
use image::EncodableLayout;

mod generate;

fn main() {
	let native_options = NativeOptions {
		initial_window_size: Some(Vec2::new(1280.0, 720.0)),
		..NativeOptions::default()
	};

	eframe::run_native(
		"Julia fractal render GUI",
		native_options,
		Box::new(|cc| Box::new(JuliaGUI::new(cc))),
	)
	.unwrap();
}

struct JuliaGUI {
	color: (u8, u8, u8),
	preview: TextureHandle,
	render_options: RenderOptions,
	preview_render_ms: f64,
	export_render_ms: f64,
	export_res_multiplier: u32,
	export_iterations: u32,
	export_name: String,
	settings_changed: bool,
}

impl JuliaGUI {
	fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let preview = cc.egui_ctx.load_texture(
			"preview_image",
			egui::ColorImage::from_rgb([1, 1], &[0, 0, 0]),
			TextureOptions::default(),
		);
		let preview_quality = RenderOptions {
			width: 512,
			height: 512,
			unit_width: 4.0,
			max_iterations: 128,
			cx: -0.981,
			cy: -0.277,
			fill_style: FillStyle::Bright,
		};

		Self {
			color: (12, 5, 10),
			preview,
			render_options: preview_quality,
			preview_render_ms: 0.0,
			export_render_ms: f64::NAN,
			export_res_multiplier: 8,
			export_iterations: 512,
			export_name: String::from("julia_set.png"),
			settings_changed: true,
		}
	}

	fn update_preview(&mut self) {
		let start_time = SystemTime::now();
		let preview = render(&self.render_options, self.color);
		self.preview.set(
			egui::ColorImage::from_rgb(
				[preview.width() as usize, preview.height() as usize],
				preview.as_bytes(),
			),
			TextureOptions::default(),
		);
		self.preview_render_ms = start_time.elapsed().unwrap().as_micros() as f64 / 1000.0;
	}

	fn export_render(&mut self) {
		let start_time = SystemTime::now();
		let settings = RenderOptions {
			width: self.render_options.width * self.export_res_multiplier,
			height: self.render_options.height * self.export_res_multiplier,
			max_iterations: self.export_iterations,
			..self.render_options.clone()
		};
		let image = render(&settings, self.color);
		if let Err(err) = image.save(&self.export_name) {
			println!("Error exporting render: {err}");
		}
		self.export_render_ms = start_time.elapsed().unwrap().as_micros() as f64 / 1000.0;
	}
}

impl eframe::App for JuliaGUI {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
		if self.settings_changed {
			self.update_preview();
			self.settings_changed = false;
		}

		egui::SidePanel::left("main_left_panel")
			.resizable(false)
			.exact_width(200.0)
			.show(ctx, |ui| {
				ui.label(RichText::new("Fractal settings").heading());
				if ui.button("Update preview").clicked() {
					self.settings_changed = true;
				}
				ui.label(format!(
					"last preview render took {:.2}ms",
					self.preview_render_ms
				));

				ui.label("CX:");
				let set_cx = ui.add(Slider::new(&mut self.render_options.cx, -2.0..=2.0));
				ui.label("CY:");
				let set_cy = ui.add(Slider::new(&mut self.render_options.cy, -2.0..=2.0));
				ui.label("Image width in space units:");
				let set_unit_width =
					ui.add(Slider::new(&mut self.render_options.unit_width, 0.01..=6.0));
				ui.label("Fill style:");
				ui.horizontal(|ui| {
					let set_black = ui.radio_value(
						&mut self.render_options.fill_style,
						FillStyle::Black,
						"Black",
					);
					let set_bright = ui.radio_value(
						&mut self.render_options.fill_style,
						FillStyle::Bright,
						"Bright",
					);
					if set_bright.changed() || set_black.changed() {
						self.settings_changed = true;
					}
				});

				ui.label("Preview iterations:");
				let set_iter = ui.add(
					Slider::new(&mut self.render_options.max_iterations, 5..=256)
						.clamp_to_range(false),
				);

				ui.label(RichText::new("Render settings").heading());
				ui.label("preview resolution:");
				ui.horizontal(|ui| {
					let set_width = ui.add(
						DragValue::new(&mut self.render_options.width).clamp_range(128..=16384),
					);
					ui.label("x");
					let set_height = ui.add(
						DragValue::new(&mut self.render_options.height).clamp_range(128..=16384),
					);
					if set_width.changed() || set_height.changed() {
						self.settings_changed = true;
					}
				});

				ui.label("Export iterations:");
				ui.add(Slider::new(&mut self.export_iterations, 5..=1024).clamp_to_range(false));
				ui.label("Resolution multiplier:");
				ui.add(Slider::new(&mut self.export_res_multiplier, 1..=32));
				ui.label(format!(
					"Export resolution: {}x{}",
					self.export_res_multiplier * self.render_options.width,
					self.export_res_multiplier * self.render_options.height
				));

				let render_button = ui.button(format!("Render to '{}'", &self.export_name));
				if render_button.clicked() {
					self.export_render();
				}
				ui.label(format!(
					"last exported render took {:.2}ms",
					self.export_render_ms
				));

				if set_cx.changed()
					|| set_cy.changed() || set_unit_width.changed()
					|| set_iter.changed()
				{
					self.settings_changed = true;
				}
			});

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.image(&self.preview, self.preview.size_vec2());
		});
	}

	fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}
}
