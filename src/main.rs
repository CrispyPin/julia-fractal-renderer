#![windows_subsystem = "windows"]
use std::{
	fs::{self, File},
	io::Write,
	path::PathBuf,
	sync::mpsc::{self, Receiver, Sender},
	thread::{self, JoinHandle},
	time::SystemTime,
};

use eframe::{
	egui::{self, DragValue, RichText, Slider, TextureOptions},
	epaint::{TextureHandle, Vec2},
	Frame, NativeOptions,
};
use generate::{render, view_point, FillStyle, RenderOptions};
use image::EncodableLayout;
use native_dialog::FileDialog;
use serde::{Deserialize, Serialize};

mod generate;

const SETTINGS_FILE: &str = "fractal_settings.json";

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

#[derive(Serialize, Deserialize)]
struct JuliaGUI {
	color: (u8, u8, u8),
	#[serde(skip)]
	preview: Option<TextureHandle>,
	settings: RenderOptions,
	#[serde(skip)]
	preview_render_ms: f64,
	#[serde(skip)]
	export_render_ms: Option<f64>,
	export_res_power: u8,
	export_iterations: u32,
	#[serde(skip)]
	export_path: PathBuf,
	#[serde(skip)]
	settings_changed: bool,
	preview_point: bool,
	#[serde(skip)]
	render_thread_handle: Option<JoinHandle<()>>,
	#[serde(skip)]
	render_thread: Option<Sender<RenderJob>>,
	#[serde(skip)]
	render_result: Option<Receiver<f64>>,
	#[serde(skip)]
	waiting: bool,
}

enum RenderJob {
	Render(PathBuf, RenderOptions, (u8, u8, u8)),
	Exit,
}

impl Default for JuliaGUI {
	fn default() -> Self {
		Self {
			color: (12, 5, 10),
			preview: None,
			settings: RenderOptions::default(),
			preview_render_ms: 0.0,
			export_render_ms: None,
			export_res_power: 3,
			export_iterations: 512,
			export_path: "".into(),
			settings_changed: true,
			preview_point: false,
			render_thread_handle: None,
			render_thread: None,
			render_result: None,
			waiting: false,
		}
	}
}

impl JuliaGUI {
	fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let mut n: Self = fs::read_to_string(SETTINGS_FILE)
			.map(|s| serde_json::from_str(&s).ok())
			.ok()
			.flatten()
			.unwrap_or_default();

		let preview = cc.egui_ctx.load_texture(
			"preview_image",
			egui::ColorImage::from_rgb([1, 1], &[0, 0, 0]),
			TextureOptions::default(),
		);

		let (job_sender, job_receiver) = mpsc::channel::<RenderJob>();
		let (result_sender, result_receiver) = mpsc::channel::<f64>();
		let render_thread = thread::Builder::new()
			.name("render".into())
			.spawn(move || loop {
				if let Ok(job) = job_receiver.recv() {
					match job {
						RenderJob::Exit => break,
						RenderJob::Render(path, options, color) => {
							let start_time = SystemTime::now();
							let image = render(&options, color);
							if let Err(err) = image.save(&path) {
								println!("Failed to save render: {err}");
							}
							let time = start_time.elapsed().unwrap().as_micros() as f64 / 1000.0;
							result_sender.send(time).unwrap();
						}
					}
				}
			})
			.unwrap();

		n.preview = Some(preview);
		n.settings_changed = true;
		n.export_path = "julia_fractal.png".into();
		n.render_thread_handle = Some(render_thread);
		n.render_thread = Some(job_sender);
		n.render_result = Some(result_receiver);
		n
	}

	fn save_settings(&self) {
		let settings = serde_json::to_string_pretty(&self).unwrap();
		let mut file = File::create(SETTINGS_FILE).unwrap();
		file.write_all(settings.as_bytes()).unwrap();
	}

	fn update_preview(&mut self) {
		let start_time = SystemTime::now();
		let mut frame = render(&self.settings, self.color);
		if self.preview_point {
			frame = view_point(&self.settings, frame);
		}

		if let Some(preview) = &mut self.preview {
			preview.set(
				egui::ColorImage::from_rgb(
					[frame.width() as usize, frame.height() as usize],
					frame.as_bytes(),
				),
				TextureOptions::default(),
			);
		}
		self.preview_render_ms = start_time.elapsed().unwrap().as_micros() as f64 / 1000.0;
	}

	fn export_render(&mut self) {
		self.save_settings();
		if let Some(channel) = &self.render_thread {
			let res_mul = 1 << self.export_res_power;
			let settings = RenderOptions {
				width: self.settings.width * res_mul,
				height: self.settings.height * res_mul,
				iterations: self.export_iterations,
				..self.settings.clone()
			};

			channel
				.send(RenderJob::Render(
					self.export_path.clone(),
					settings,
					self.color,
				))
				.expect("failed to start render job");
			self.waiting = true;
		}
	}

	fn export_render_new_path(&mut self) {
		if let Ok(Some(path)) = FileDialog::new()
			.set_filename(&self.export_path.to_string_lossy())
			.add_filter("PNG file", &["png"])
			.show_save_single_file()
		{
			self.export_path = path;
			self.export_render();
		}
	}
}

impl eframe::App for JuliaGUI {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
		if self.settings_changed {
			self.update_preview();
			self.save_settings();
			self.settings_changed = false;
		}

		if let Some(result) = self.render_result.as_mut().and_then(|r| r.try_recv().ok()) {
			self.export_render_ms = Some(result);
			self.waiting = false;
		}

		egui::SidePanel::left("main_left_panel")
			.resizable(false)
			.exact_width(200.0)
			.show(ctx, |ui| {
				ui.label(format!(
					"Preview render took {:.2}ms",
					self.preview_render_ms
				));

				let set_point_vis = ui.checkbox(&mut self.preview_point, "View C point");
				ui.label("C point (X, Y):");
				let set_cx =
					ui.add(Slider::new(&mut self.settings.cx, -1.0..=1.0).clamp_to_range(false));
				let set_cy =
					ui.add(Slider::new(&mut self.settings.cy, -1.0..=1.0).clamp_to_range(false));
				ui.label("render width:");
				let set_unit_width = ui.add(Slider::new(&mut self.settings.unit_width, 0.1..=6.0));
				ui.label("Fill style:");
				ui.horizontal(|ui| {
					let set_black =
						ui.radio_value(&mut self.settings.fill_style, FillStyle::Black, "Black");
					let set_bright =
						ui.radio_value(&mut self.settings.fill_style, FillStyle::Bright, "Bright");
					if set_bright.changed() || set_black.changed() {
						self.settings_changed = true;
					}
				});

				ui.horizontal(|ui| {
					ui.label("Colour (RGB)");
					ui.menu_button("presets", |ui| {
						if ui.button("pink").clicked() {
							self.color = (8, 2, 6);
							self.settings_changed = true;
						}
						if ui.button("blue").clicked() {
							self.color = (2, 4, 8);
							self.settings_changed = true;
						}
						if ui.button("green").clicked() {
							self.color = (2, 8, 4);
							self.settings_changed = true;
						}
						if ui.button("salmon").clicked() {
							self.color = (8, 4, 4);
							self.settings_changed = true;
						}
						if ui.button("purple").clicked() {
							self.color = (5, 2, 11);
							self.settings_changed = true;
						}
						if ui.button("randomise").clicked() {
							self.color = (
								rand::random::<u8>() & 15,
								rand::random::<u8>() & 15,
								rand::random::<u8>() & 15,
							);
							self.settings_changed = true;
						}
					});
				});
				let set_red = ui.add(Slider::new(&mut self.color.0, 0..=16));
				let set_green = ui.add(Slider::new(&mut self.color.1, 0..=16));
				let set_blue = ui.add(Slider::new(&mut self.color.2, 0..=16));

				ui.label("Preview iterations:");
				let set_iter = ui
					.add(Slider::new(&mut self.settings.iterations, 5..=256).clamp_to_range(false));

				ui.label("Preview resolution:");
				ui.horizontal(|ui| {
					let set_width = ui.add(
						DragValue::new(&mut self.settings.width)
							.clamp_range(128..=2048)
							.suffix("px"),
					);
					ui.label("x");
					let set_height = ui.add(
						DragValue::new(&mut self.settings.height)
							.clamp_range(128..=2048)
							.suffix("px"),
					);
					if set_width.changed() || set_height.changed() {
						self.settings_changed = true;
					}
				});
				ui.menu_button("presets", |ui| {
					if ui.button("1:1 512x512").clicked() {
						self.settings.width = 512;
						self.settings.height = 512;
						self.settings_changed = true;
					}
					if ui.button("16:9 960x540 (half hd)").clicked() {
						self.settings.width = 960;
						self.settings.height = 540;
						self.settings_changed = true;
					}
					if ui.button("2:1 1024x512").clicked() {
						self.settings.width = 1024;
						self.settings.height = 512;
						self.settings_changed = true;
					}
				});

				ui.label("Render iterations:");
				ui.add(Slider::new(&mut self.export_iterations, 5..=1024).clamp_to_range(false));
				ui.label("Render resolution:");
				ui.add(Slider::new(&mut self.export_res_power, 0..=6).clamp_to_range(false));
				ui.label(format!(
					"Render resolution: {}x{}",
					(1 << self.export_res_power) * self.settings.width,
					(1 << self.export_res_power) * self.settings.height
				));

				ui.horizontal(|ui| {
					ui.add_enabled_ui(!self.waiting, |ui| {
						let export_text = if self.export_path.is_file() {
							"Overwrite"
						} else {
							"Render"
						};
						if ui.button(export_text).clicked() {
							self.export_render();
						}
						if ui.button("Render to").clicked() {
							self.export_render_new_path();
						}
					});
					if self.waiting {
						ui.spinner();
					}
				});
				ui.label(
					self.export_path
						.file_name()
						.unwrap_or_default()
						.to_string_lossy()
						.to_string(),
				);

				let predicted_render_time = (self.preview_render_ms
					* (1 << self.export_res_power * 2) as f64
					* (self.export_iterations as f64 / self.settings.iterations as f64)
					/ 1000.0)
					.floor();
				if predicted_render_time < 60.0 {
					ui.label(format!("Predicted render time: {predicted_render_time}s"));
				} else {
					ui.label(format!(
						"Predicted render time: {:.1} min",
						predicted_render_time / 60.0
					));
				}

				if let Some(ms) = self.export_render_ms {
					ui.label(format!("(took {ms:.2}ms)"));
				}

				if set_cx.changed()
					|| set_cy.changed() || set_unit_width.changed()
					|| set_iter.changed()
					|| set_red.changed() || set_green.changed()
					|| set_blue.changed()
					|| set_point_vis.changed()
				{
					self.settings_changed = true;
				}
			});

		egui::CentralPanel::default().show(ctx, |ui| {
			if let Some(texture) = &self.preview {
				ui.image(texture, texture.size_vec2());
			}
		});
	}

	fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
		if let Some(channel) = &self.render_thread {
			channel.send(RenderJob::Exit).unwrap();
		}
		self.render_thread_handle.take().unwrap().join().unwrap();
		self.save_settings();
	}
}
