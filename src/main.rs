use eframe::egui;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::Nvml;

struct GpuData{
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
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            gpu_data: GpuData{
                name: "No Name Data".to_string(),
                memory_total: 0, //"No Memory Data".to_string(),
                memory_used: 0,//"No Memory Data".to_string(),
                temperature: 0,//"No Temperature Data".to_string(),
                utilization: "No Utiization Data".to_string(),
            },
            animate_memory_bar: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            //populates gpu_data with gpu information
            let nvml = Nvml::init().expect("NVML failed to initialize"); //make this not exit the program
            let device_wrapped = nvml.device_by_index(0);
            let device = device_wrapped.unwrap();

            let curr_gpudata = GpuData{
                name: device.name().unwrap(),
                memory_used: device.memory_info().unwrap().used/1024/1024,
                memory_total: device.memory_info().unwrap().total/1024/1024,
                temperature: device.temperature(TemperatureSensor::Gpu).unwrap(),
                utilization: device.encoder_utilization().unwrap().utilization.to_string(),
            };
            self.gpu_data = curr_gpudata;

            let memory_util = ((((self.gpu_data.memory_used as f64/ self.gpu_data.memory_total as f64)*100.0) * 100.0).round())/100.0;

            //Start of ui being built

            ui.heading("GPU Stats");

            //Collapsable menu that shows all the condensed stats
            ui.horizontal(|ui| {
                ui.collapsing("All stats", |ui|{
                    ui.label(&self.gpu_data.name);
                    ui.label("Memory used: ".to_owned() + &self.gpu_data.memory_used.to_string() + "MB");
                    ui.label("Memory total: ".to_owned() + &self.gpu_data.memory_total.to_string() + "MB");
                    ui.label("Memory Utilization: ".to_owned() + memory_util.to_string().as_str() + "%");
                    ui.label("Encoder Utilization: ".to_owned() + &self.gpu_data.utilization + "%");
                    ui.label("Temperature: ".to_owned() + &self.gpu_data.temperature.to_string() + "°C");
                });
            });

            let insert_text = self.gpu_data.memory_used.to_string() + "MB/" +self.gpu_data.memory_total.to_string().as_str() + "MB";
            //let animate_annotation = ui.add(progress).on_hover_text(insert_text);
            ui.label("Memory Usage");
            //let used = (self.gpu_data.memory_used/self.gpu_data.memory_total) as f32;
            let memory_bar = egui::ProgressBar::new(memory_util as f32/100.0)
            .show_percentage()
            .animate(self.animate_memory_bar);
            self.animate_memory_bar = ui
            .add(memory_bar)
            .on_hover_text(insert_text)
            .hovered();

            ui.vertical(|ui|{
                ui.label("Thermometer");
            });
        });
    }
}