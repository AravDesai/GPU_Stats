#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use core::time;
use std::thread;
use std::process::Command;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use eframe::egui;

struct GpuData{
    name: String,
    memory_total: u64,
    memory_used: u64,
    temperature: u64,
    utilization: u64,
    fan_speed: String
}

fn updater(update_timer: Duration){
    let python_command = if cfg!(windows) {
        "python"
    } else {
        "python3"
    };

    let python_script = Command::new(python_command)
        .arg("gpustats.py")
        .status()
        .expect("Failed to run Python code");

    if python_script.success() {
        let contents = fs::read_to_string("gpuinfo.txt")
            .expect("Failed to read file");
        let lines: Vec<&str> = contents.split('\n').collect();

        let curr_gpudata = GpuData {
            name: lines[0].to_string(),
            memory_total: lines[1].parse().unwrap(),
            memory_used: lines[2].parse().unwrap(),
            temperature: lines[3].parse().unwrap(),
            utilization: lines[4].parse().unwrap(),
            fan_speed: lines[5].to_string()
        };

    } 
    else {
        panic!("Python script failed to execute");
    }
    sleep(update_timer);
    updater(update_timer);
}

fn main() {
    let mut threads = vec![];
    let new_thread = thread::spawn(move ||{
        let dur: Duration = time::Duration::from_secs(5);
        updater(dur);
    });
    threads.push(new_thread);

    
    for th in threads{
        th.join().unwrap();
    }
}
