use eframe::egui;
use egui::{Color32, ComboBox, Pos2, Response};
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::Nvml;
use std::thread;
use std::time::Duration;

struct GpuData {
    name: String,
    memory_total: u64,
    memory_used: u64,
    temperature: u32,
    utilization: String,
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 300.0]),
        ..Default::default()
    };
    eframe::run_native(
        "GPU stats",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    gpu_data: GpuData,
    animate_memory_bar: bool,
    animate_thermometer_bar: bool,
    c_to_f_indexer: usize,
    update_blocker: bool,
    //tester: u32,
    nvml: Nvml,
    special_temp: u32,
    device_indexer: u32,
    fan_indexer: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            gpu_data: GpuData {
                name: "No Name Data".to_string(),
                memory_total: 0,
                memory_used: 0,
                temperature: 0,
                utilization: "No Utiization Data".to_string(),
            },
            animate_memory_bar: false,
            animate_thermometer_bar: false,
            c_to_f_indexer: 0,
            update_blocker: true,
            //tester: 0
            nvml: Nvml::init().expect("NVML failed to initialize"), // Make this not crash for non nvidia systems/non gpu computers (could be implemented in the update function)
            special_temp: 0,
            device_indexer: 0,
            fan_indexer: 0,
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
    return Color32::from_rgb(red as u8, 0, blue as u8);
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //populates gpu_data with gpu information

            let device_wrapped = self.nvml.device_by_index(self.device_indexer);
            let device = device_wrapped.unwrap();

            self.gpu_data = GpuData {
                name: device.name().unwrap(),
                memory_used: device.memory_info().unwrap().used / 1024 / 1024,
                memory_total: device.memory_info().unwrap().total / 1024 / 1024,
                temperature: device.temperature(TemperatureSensor::Gpu).unwrap(),
                utilization: device
                    .encoder_utilization()
                    .unwrap()
                    .utilization
                    .to_string(),
            };

            //Makes the window autoupdate without user activity
            if self.update_blocker {
                self.special_temp = self.gpu_data.temperature;
                let ctx2 = ctx.clone();
                self.update_blocker = false;
                thread::spawn(move || loop {
                    ctx2.request_repaint();
                    thread::sleep(Duration::from_millis(500));
                });
            }

            let memory_util = ((((self.gpu_data.memory_used as f64
                / self.gpu_data.memory_total as f64)
                * 100.0)
                * 100.0)
                .round())
                / 100.0;

            let c_to_f = ["°C", "°F"];
            let curr_temp_type = c_to_f[self.c_to_f_indexer];

            //Start of ui being built
            ui.heading("GPU Stats");

            ui.horizontal(|ui| {
                //Collapsable menu that shows all the condensed stats

                let holder = ui.collapsing("All stats", |ui| {
                    ui.label(&self.gpu_data.name);
                    ui.label(
                        "Memory used: ".to_owned() + &self.gpu_data.memory_used.to_string() + "MB",
                    );
                    ui.label(
                        "Memory total: ".to_owned()
                            + &self.gpu_data.memory_total.to_string()
                            + "MB",
                    );
                    ui.label(
                        "Memory Utilization: ".to_owned() + memory_util.to_string().as_str() + "%",
                    );
                    ui.label("Encoder Utilization: ".to_owned() + &self.gpu_data.utilization + "%");
                    ui.label(
                        "Temperature: ".to_owned()
                            + &self.special_temp.to_string()
                            + curr_temp_type,
                    );
                });

                //Config menu

                let holder_rect = egui::Rect {
                    min: Pos2 {
                        x: holder.header_response.rect.min.x + 250.0,
                        y: holder.header_response.rect.min.y,
                    },
                    max: holder.header_response.rect.max,
                };

                let mut devices = vec![];

                if let Ok(i) = self.nvml.device_count() {
                    devices.push(self.nvml.device_by_index(i - 1).unwrap().name().unwrap());
                }

                let mut fans = vec![];

                if let Ok(i) = self
                    .nvml
                    .device_by_index(self.device_indexer)
                    .unwrap()
                    .num_fans()
                {
                    fans.push(i);
                }

                ui.allocate_ui_at_rect(holder_rect, |ui| {
                    ui.collapsing("Configurations", |ui| {
                        egui::ComboBox::from_label("GPU Picker")
                            .selected_text(format!("{devices:?}"))
                            .show_ui(ui, |ui| {
                                let mut indexer = 0;
                                for selectable in devices {
                                    if ui
                                        .selectable_value(
                                            &mut device.name().unwrap(),
                                            selectable.clone(),
                                            device.name().unwrap(),
                                        )
                                        .clicked()
                                    {
                                        self.device_indexer = indexer;
                                    }
                                    indexer += 1;
                                }
                            });

                        egui::ComboBox::from_label("Fan Picker")
                            .selected_text(format!("Fan {}", self.fan_indexer))
                            .show_ui(ui, |ui| {
                                for (index, fan) in fans.iter().enumerate() {
                                    let indexer = index as u32;
                                    if ui
                                        .selectable_value(
                                            &mut self.fan_indexer,
                                            indexer,
                                            format!("Fan {}", index),
                                        )
                                        .clicked()
                                    {
                                        self.fan_indexer = indexer;
                                    }
                                }
                            });
                    });
                });
            });

            //Memory bar code
            let insert_memory_text = self.gpu_data.memory_used.to_string()
                + "MB/"
                + self.gpu_data.memory_total.to_string().as_str()
                + "MB";
            ui.label("Memory Usage");
            let memory_bar = egui::ProgressBar::new(memory_util as f32 / 100.0)
                .show_percentage()
                .animate(self.animate_memory_bar);
            self.animate_memory_bar = ui
                .add(memory_bar)
                .on_hover_text(insert_memory_text)
                .hovered();

            //Testing bar
            //ui.add(egui::Slider::new(&mut self.tester, 0..=100).text("Testing Bar"));

            //Thermometer
            ui.label(
                "Thermometer
            ",
            ); //Added newline for space

            //Bar portion of thermometer
            let thermometer = egui::ProgressBar::new(self.gpu_data.temperature as f32 / 100.0)
                .fill(color_gradient(self.gpu_data.temperature))
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
                    color_gradient(self.gpu_data.temperature),
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
                self.special_temp = ((9 * self.gpu_data.temperature) / 5) + 32;
            } else {
                self.special_temp = self.gpu_data.temperature;
            }

            //Fan Speed Info
            ui.label(
                "
            ",
            ); //more empty space

            let fan_speed = match device.fan_speed(self.fan_indexer) {
                Ok(speed) => speed,
                Err(_E) => 0,
            };

            ui.label("Number of Fans: ".to_owned() + &device.num_fans().unwrap().to_string());
            ui.label("Fan speed: ".to_owned() + &fan_speed.to_string());
        });
    }
}
