use eframe::egui;
use egui::{Color32, Pos2};
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
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
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
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
            let device_wrapped = self.nvml.device_by_index(0);
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

            //Collapsable menu that shows all the condensed stats
            ui.horizontal(|ui| {
                ui.collapsing("All stats", |ui| {
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

            //Hovering thermometer
            // self.animate_thermometer_bar = ui
            //     .add(thermometer)
            //     .on_hover_text(
            //         self.gpu_data.temperature.to_string().as_str().to_owned() + curr_temp_type,
            //     )
            //     .hovered();

            let thermometer_rect = ui.add(thermometer).rect;

            //let therm_rect = ;

            // Bulb portion of thermometer

            //let rect = egui::Rect::from_min_max(Pos2{x: 80.0, y:80.0}, Pos2{x:280.0, y:280.0});

            ui.allocate_ui_at_rect(thermometer_rect, |ui| {
                let painter = ui.painter();
                painter.circle(
                    Pos2 {
                        x: thermometer_rect.min.x + 15.0,
                        y: thermometer_rect.min.y + 8.0,
                    },
                    19.0,
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
        });
    }
}
