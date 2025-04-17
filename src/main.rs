#![windows_subsystem = "windows"]

use eframe::egui::{self, Context};
use egui::{Color32, Pos2};
use egui_gauge::Gauge;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::Nvml;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Default, Clone, Copy)]
struct Stat {
    memory_used: u64,
    temperature: u32,
    utilization: u32,
    fan_speed: u32,
}

struct GpuData {
    name: String,
    memory_total: u64,
    history: Vec<Stat>,
    num_fans: u32,
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "GPU stats",
        options,
        Box::new(|cc| Box::new(MyApp::init(cc.egui_ctx.clone()))),
    )
}

struct MyApp {
    gpu_data: Vec<GpuData>,
    animate_memory_bar: bool,
    animate_thermometer_bar: bool,
    c_to_f_indexer: usize,
    device_idx: usize,
    fan_idx: usize,
    number_of_datapoints: usize,
    stat_rx: mpsc::Receiver<Vec<Stat>>,
    special_temp: u32,
}

impl MyApp {
    fn init(ctx: Context) -> Self {
        let nvml = Nvml::init().expect("NVML failed to initialize");
        let device_count = nvml.device_count().unwrap();

        let mut gpu_data = vec![];

        for device in 0..device_count {
            let device = nvml.device_by_index(device).unwrap();
            gpu_data.push(GpuData {
                name: device.name().unwrap(),
                memory_total: device.memory_info().unwrap().total / 1024 / 1024,
                history: vec![],
                num_fans: device.num_fans().unwrap(),
            });
        }

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || loop {
            let mut stat_data = vec![];
            for device in 0..device_count {
                let device = nvml.device_by_index(device).unwrap();

                stat_data.push(Stat {
                    memory_used: device.memory_info().unwrap().used / 1024 / 1024,
                    temperature: device.temperature(TemperatureSensor::Gpu).unwrap(),
                    utilization: device.encoder_utilization().unwrap().utilization,
                    fan_speed: device.fan_speed(0).unwrap_or_default(),
                });
            }
            tx.send(stat_data).unwrap();
            ctx.request_repaint();
            thread::sleep(Duration::from_millis(500));
        });
        Self {
            stat_rx: rx,
            gpu_data,
            animate_memory_bar: false,
            animate_thermometer_bar: false,
            c_to_f_indexer: 0,
            device_idx: 0,
            fan_idx: 0,
            number_of_datapoints: 10,
            special_temp: 0,
        }
    }
}

fn color_gradient(temperature: u32) -> Color32 {
    let mut blue: i32 = 255 - (2 * (temperature) + 44) as i32;
    let mut red = (2 * temperature) + 88;
    if red > 255 {
        red = 255;
    }
    if blue < 0 {
        blue = 0;
    }
    Color32::from_rgb(red as u8, 0, blue as u8)
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let received = self.stat_rx.try_recv();

            if let Ok(stats) = received {
                for (idx, stat) in stats.into_iter().enumerate() {
                    self.gpu_data[idx].history.push(stat);
                }
            }

            let last_stat = self.gpu_data[self.device_idx]
                .history
                .iter()
                .last()
                .copied()
                .unwrap_or_default();
            let memory_util = ((((last_stat.memory_used as f64
                / self.gpu_data[self.device_idx].memory_total as f64)
                * 100.0)
                * 100.0)
                .round())
                / 100.0;

            let c_to_f = ["°C", "°F"];
            let curr_temp_type = c_to_f[self.c_to_f_indexer];

            //Start of ui being built
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("GPU Stats");

                ui.horizontal(|ui| {
                    //Collapsable menu that shows all the condensed stats

                    let holder = ui.collapsing("All stats", |ui| {
                        ui.label(&self.gpu_data[self.device_idx].name);
                        ui.label(
                            "Memory used: ".to_owned() + &last_stat.memory_used.to_string() + "MB",
                        );
                        ui.label(
                            "Memory total: ".to_owned()
                                + &self.gpu_data[self.device_idx].memory_total.to_string()
                                + "MB",
                        );
                        ui.label(
                            "Memory Utilization: ".to_owned()
                                + memory_util.to_string().as_str()
                                + "%",
                        );
                        ui.label(
                            "Encoder Utilization: ".to_owned()
                                + &last_stat.utilization.to_string()
                                + "%",
                        );
                        ui.label(
                            "Temperature: ".to_owned()
                                + &last_stat.temperature.to_string()
                                + curr_temp_type,
                        );
                        ui.label(
                            "Number of Fans: ".to_owned()
                                + &self.gpu_data[self.device_idx].num_fans.to_string(),
                        );
                        ui.label("Fan Speed: ".to_owned() + &last_stat.fan_speed.to_string() + "%");
                    });

                    //Config menu

                    let holder_rect = egui::Rect {
                        min: Pos2 {
                            x: holder.header_response.rect.min.x + 250.0,
                            y: holder.header_response.rect.min.y,
                        },
                        max: holder.header_response.rect.max,
                    };

                    let mut device_names = vec![];

                    for idx in 0..self.gpu_data.len() {
                        device_names.push(self.gpu_data[idx].name.clone())
                    }

                    ui.allocate_ui_at_rect(holder_rect, |ui| {
                        ui.collapsing("Configurations", |ui| {
                            egui::ComboBox::from_label("GPU Picker")
                                .selected_text(format!("{device_names:?}"))
                                .show_ui(ui, |ui| {
                                    let mut indexer = 0;
                                    for selectable in device_names {
                                        if ui
                                            .selectable_value(
                                                &mut self.gpu_data[self.device_idx].name,
                                                selectable.clone(),
                                                selectable,
                                            )
                                            .clicked()
                                        {
                                            self.device_idx = indexer;
                                        }
                                        indexer += 1;
                                    }
                                });

                            let mut fans: Vec<String> = vec![];

                            for idx in 0..self.gpu_data[self.device_idx].num_fans {
                                fans.push(idx.to_string())
                            }

                            egui::ComboBox::from_label("Fan Picker")
                                .selected_text(format!("Fan {}", self.fan_idx))
                                .show_ui(ui, |ui| {
                                    for (index, _fan) in fans.iter().enumerate() {
                                        let indexer = index as u32;
                                        if ui
                                            .selectable_value(
                                                &mut self.fan_idx,
                                                indexer as usize,
                                                format!("Fan {}", index),
                                            )
                                            .clicked()
                                        {
                                            self.fan_idx = indexer as usize;
                                        }
                                    }
                                });
                        });
                    });
                });

                //Memory bar code
                let insert_memory_text = last_stat.memory_used.to_string()
                    + "MB/"
                    + self.gpu_data[self.device_idx]
                        .memory_total
                        .to_string()
                        .as_str()
                    + "MB";
                ui.label("Memory Usage");
                let memory_bar = egui::ProgressBar::new(memory_util as f32 / 100.0)
                    .show_percentage()
                    .animate(self.animate_memory_bar);
                self.animate_memory_bar = ui
                    .add(memory_bar)
                    .on_hover_text(insert_memory_text)
                    .hovered();

                //Thermometer
                ui.label("Thermometer\n");

                //Bar portion of thermometer
                let thermometer = egui::ProgressBar::new(last_stat.temperature as f32 / 100.0)
                    .fill(color_gradient(last_stat.temperature))
                    .animate(self.animate_thermometer_bar);

                // Bulb portion of thermometer

                let thermometer_rect = ui.add(thermometer).rect;

                ui.allocate_ui_at_rect(thermometer_rect, |ui| {
                    let painter = ui.painter();
                    painter.circle(
                        Pos2 {
                            x: thermometer_rect.min.x + 13.0,
                            y: thermometer_rect.min.y + 8.0,
                        },
                        20.0,
                        color_gradient(last_stat.temperature),
                        egui::Stroke {
                            width: 0.0,
                            color: Color32::from_rgb(255, 255, 255),
                        },
                    );

                    let temp_changer = ui
                        .button(self.special_temp.to_string() + curr_temp_type)
                        .on_hover_text("Click to change unit");
                    if temp_changer.clicked() {
                        self.c_to_f_indexer = if self.c_to_f_indexer == 0 { 1 } else { 0 };
                    }
                });

                if self.c_to_f_indexer == 1 {
                    self.special_temp = ((9 * last_stat.temperature) / 5) + 32;
                } else {
                    self.special_temp = last_stat.temperature;
                }

                //Fan Speed Info

                ui.label("\nFan Speed");

                ui.add(
                    Gauge::new(last_stat.fan_speed, 0..=100, 200.0, Color32::BLUE)
                        .text("Fan Speed%"),
                );

                ui.collapsing("Graphs", |ui| {
                    let data_point_slider =
                        egui::Slider::new(&mut self.number_of_datapoints, 1..=100)
                            .text("Data Points");

                    ui.add(data_point_slider);

                    while self.gpu_data[self.device_idx].history.len() > self.number_of_datapoints {
                        self.gpu_data[self.device_idx].history.remove(0);
                    }

                    ui.label("Memory Graph");

                    egui_plot::Plot::new("Memory Graph")
                        .allow_drag(false)
                        .allow_scroll(false)
                        .allow_zoom(false)
                        .x_axis_label("Time")
                        .include_y(0.0)
                        .y_axis_label("Memory in use")
                        .height(500.0)
                        .show(ui, |plot_ui| {
                            let memory_points = egui_plot::PlotPoints::from_ys_f32(
                                &self.gpu_data[self.device_idx]
                                    .history
                                    .iter()
                                    .map(|s| s.memory_used as f32)
                                    .collect::<Vec<f32>>(),
                            );
                            plot_ui.line(
                                egui_plot::Line::new(memory_points)
                                    .fill(0.0)
                                    .color(Color32::BLUE),
                            );
                            plot_ui.set_auto_bounds(true.into());
                        });

                    ui.label("Temperature Graph");

                    egui_plot::Plot::new("Temperature Graph")
                        .allow_drag(false)
                        .allow_scroll(false)
                        .allow_zoom(false)
                        .x_axis_label("Time")
                        .include_y(40.0)
                        .height(500.0)
                        .y_axis_label("Temperature")
                        .show(ui, |plot_ui| {
                            let temperature_points = egui_plot::PlotPoints::from_ys_f32(
                                &self.gpu_data[self.device_idx]
                                    .history
                                    .iter()
                                    .map(|s| s.temperature as f32)
                                    .collect::<Vec<f32>>(),
                            );
                            plot_ui.line(
                                egui_plot::Line::new(temperature_points)
                                    .fill(0.0)
                                    .color(Color32::RED),
                            );

                            plot_ui.set_auto_bounds(true.into());
                        });
                })
            });
        });
    }
}
