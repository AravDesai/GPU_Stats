use eframe::egui;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::Nvml;
use egui::Color32;

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
    animate_thermometer_bar: bool,
    c_to_f_indexer:usize,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            gpu_data: GpuData{
                name: "No Name Data".to_string(),
                memory_total: 0,
                memory_used: 0,
                temperature: 0,
                utilization: "No Utiization Data".to_string(),
            },
            animate_memory_bar: false,
            animate_thermometer_bar:false,
            c_to_f_indexer: 0,
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


            //Memory bar code
            let insert_memory_text = self.gpu_data.memory_used.to_string() + "MB/" +self.gpu_data.memory_total.to_string().as_str() + "MB";
            ui.label("Memory Usage");
            let memory_bar = egui::ProgressBar::new(memory_util as f32/100.0)
            .show_percentage()
            .animate(self.animate_memory_bar);
            self.animate_memory_bar = ui
            .add(memory_bar)
            .on_hover_text(insert_memory_text)
            .hovered();

            //Thermometer
            let c_to_f = ["°C", "°F"];
            let curr_temp_type = c_to_f[self.c_to_f_indexer];

            ui.label("Thermometer");
            let thermometer = egui::ProgressBar::new(self.gpu_data.temperature as f32/100.0)
            .fill(Color32::from_rgb(255, 0, 0))
            .animate(self.animate_thermometer_bar);
            self.animate_thermometer_bar = ui
            .add(thermometer)
            .on_hover_text(self.gpu_data.temperature.to_string().as_str().to_owned() + curr_temp_type)
            .hovered();

            //image of circle/draw filled circle with red
            //text on circle with current temperature
            //can click on the text as a button which changes from C to F
        });
    }
}