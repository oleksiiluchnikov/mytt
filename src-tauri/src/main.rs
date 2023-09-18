// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use open;
use tilde_expand::tilde_expand;
mod stopwatch;

#[tauri::command]
fn start_timer(stopwatch: tauri::State<Mutex<stopwatch::Stopwatch>>) {
    if let Ok(mut stopwatch) = stopwatch.lock() {
        stopwatch.start();
    }
}

#[tauri::command]
fn stop_timer(stopwatch: tauri::State<Mutex<stopwatch::Stopwatch>>) {
    if let Ok(mut stopwatch) = stopwatch.lock() {
        stopwatch.stop();
    }
}

#[tauri::command]
fn get_time(stopwatch: tauri::State<Mutex<stopwatch::Stopwatch>>) -> String {
    if let Ok(stopwatch) = stopwatch.lock() {
        println!("{}", stopwatch.format_time());
        return stopwatch.format_time();
    }
    "Error".to_string()
}

#[tauri::command]
fn pause_timer(stopwatch: tauri::State<Mutex<stopwatch::Stopwatch>>) {
    let mut stopwatch = stopwatch.lock().unwrap();
    stopwatch.pause();
}

#[tauri::command]
fn resume_timer(stopwatch: tauri::State<Mutex<stopwatch::Stopwatch>>) {
    let mut stopwatch = stopwatch.lock().unwrap();
    stopwatch.resume();
}

#[tauri::command]
fn reset_timer(stopwatch: tauri::State<Mutex<stopwatch::Stopwatch>>) {
    let mut stopwatch = stopwatch.lock().unwrap();
    stopwatch.reset();
}

#[tauri::command]
fn submit_to_obsidian(stopwatch: tauri::State<Mutex<stopwatch::Stopwatch>>) -> Result<(), String> {
    let mut stopwatch = stopwatch.lock().map_err(|e| e.to_string())?;
    let time = stopwatch.format_time();
    let now = chrono::Local::now();
    
    let date = now.format("%Y%m%d").to_string();
    let current_time = now.format("%H%M%S").to_string();
    let file_name = format!("{}{} - {}.md", date, current_time, time).replace(":", "");
    let obsidian_url = format!("obsidian://new?vault=Knowledge&name={}", file_name);

    // Use Rust's native libraries for cross-platform command execution
    if open::that(obsidian_url).is_err() {
        return Err("Failed to open Obsidian".into());
    }
    stopwatch.stop();
    Ok(())
}


// Fetch list of notes from Obsidian Vault (~/Knowledge)
// where it has tag #Dashboard

fn get_notes() -> Result<(), String> {
    let query = "Dashboard";
    let vault_path = "~/Knowledge";
    // let not    let expanded = tilde_expand("~/Knowledge/".as_bytes());
    // for entry in std::fs::read_dir(vault_path).map_err(|e| e.to_string())? {
    //     // Print list of files in the vault
    //     let entry = entry.map_err(|e| e.to_string())?;
    //     let path = entry.path();
    //     let file_name = path.file_name().unwrap().to_str().unwrap();
    //     // get file content
    //     let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    //     // check if file contains the query
    //     // if yes, add to the list
    //     // if no, continue
    //     if content.contains(query) {
    //         println!("{} contains {}", file_name, query);
    //     }
    // }
    //
    // Most effecient way to do this is to use ripgrep
    // ripgrep -n -g "*.md" "Dashboard" ~/Knowledge
    // -n - print line number
    // -g - file glob
    // -i - case insensitive
    let output = std::process::Command::new("rg")
        .arg("-n")
        .arg("-g")
        .arg("*.md")
        .arg("-i")
        .arg(query)
        .arg(vault_path)
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    println!("{}", stdout);
    Ok(())
}


#[tauri::command]
fn get_frontmost_application() -> Result<String, String> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to get name of first application process whose frontmost is true")
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    println!("{}", stdout);
    Ok(stdout)
}

fn main() {
    let stopwatch = Mutex::new(stopwatch::Stopwatch::new());
    get_frontmost_application();

    tauri::Builder::default()
    .manage(stopwatch)
    .invoke_handler(tauri::generate_handler![
                    start_timer,
                    stop_timer,
                    get_time,
                    reset_timer,
                    pause_timer,
                    resume_timer,
                    submit_to_obsidian,
                    get_frontmost_application,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");

}
