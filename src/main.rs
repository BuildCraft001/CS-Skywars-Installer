use std::io::{self, Write};
use std::process::Command;
use std::{fs, thread, time::Duration};
use std::path::Path;
use std::fs::metadata;

use dirs;

use reqwest::blocking::get;
use serde_json::Value;

fn main() {
    println!("Welcome to the CS Skywars Installer!\n\nFirst, let's check what's the latest version...");

    let url = "https://cs-studios.net/skywars/download/get_latest_version.php";
    if let Some(file_name) = get_latest_mod_version(url) {
        let version = extract_version(&file_name);
        println!("Latest version: {}\n", version);

        print!("Are you using Minecraft Forge with the standard Minecraft launcher? (yes/no): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input == "yes" {
            let mods_folder = dirs::data_dir()
                .unwrap()
                .join(".minecraft/mods");

            let mod_path = mods_folder.join(&file_name);
            let optifine_file = "OptiFine_1.19.4_HD_U_I4.jar";
            let optifine_path = mods_folder.join(optifine_file);

            if !mods_folder.exists() {
                println!("\nMods folder not found. Creating it now...");
                fs::create_dir_all(&mods_folder).unwrap();
            }

            if mod_path.exists() {
                println!("\nThe latest mod version is already installed.");
            } else {
                println!("\nThe latest mod version is not installed.");
                print!("Do you want to install the latest version? (yes/no): ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim().to_lowercase();

                if input == "yes" {
                    let unused_mods_folder = mods_folder.parent().unwrap().join("mods_unused");
                    fs::create_dir_all(&unused_mods_folder).unwrap();

                    for entry in fs::read_dir(&mods_folder).unwrap() {
                        let entry = entry.unwrap();
                        let path = entry.path();
                        if path.is_file() && path.file_name().unwrap() != optifine_file {
                            let dest = unused_mods_folder.join(path.file_name().unwrap());
                            fs::rename(&path, &dest).unwrap();
                        }
                    }

                    println!("Other mods have been moved to 'mods_unused'.");
                    println!("Downloading latest mod...");

                    let mod_url = format!("https://cs-studios.net/skywars/download/{}", file_name);
                    let mod_data = get(&mod_url).unwrap().bytes().unwrap();
                    fs::write(&mod_path, &mod_data).unwrap();

                    println!("Latest mod installed successfully.\n");
                }
            }

            if !optifine_path.exists() {
                println!("Seems like OptiFine is not installed in Forge. It's not necessarily needed, but we still recommend it for performance reasons.");
                print!("Do you want to install OptiFine? (yes/no): ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim().to_lowercase();

                if input == "yes" {
                    println!("Opening OptiFine download page...");
                    println!("You need to scroll down to Minecraft 1.19.4 and download OptiFine HD U I4.");
                    Command::new("cmd")
                        .args(["/C", "start", "https://optifine.net/downloads"])
                        .spawn()
                        .unwrap();

                    let downloads_folder = dirs::download_dir().unwrap();
                    let optifine_download_path = downloads_folder.join(optifine_file);

                    println!("Waiting for OptiFine to appear in the Downloads folder...");
                    wait_for_download(optifine_download_path.to_str().unwrap());

                    println!("OptiFine found!");
                    fs::rename(&optifine_download_path, &optifine_path).unwrap();
                    println!("OptiFine has been moved to the mods folder.\n");
                }
            }

            let forge_base_path = dirs::data_dir()
                .unwrap()
                .join(".minecraft/versions");

            let forge_installed = forge_base_path
                .read_dir()
                .unwrap()
                .filter_map(|entry| entry.ok())
                .any(|entry| {
                    let path = entry.path();
                    let forge_jar = path.join(format!(
                        "{}.jar",
                        path.file_name().unwrap().to_str().unwrap()
                    ));
                    path.is_dir() && path.file_name().unwrap().to_str().unwrap().starts_with("1.19.4-forge-45.") && forge_jar.exists()
                });

            if !forge_installed {
                println!("Forge for 1.19.4 is not installed, but strictly needed.");
                println!("Opening Forge download page...");
                Command::new("cmd")
                    .args(["/C", "start", "https://files.minecraftforge.net/net/minecraftforge/forge/index_1.19.4.html"])
                    .spawn()
                    .unwrap();
            } else {
                println!("Seems like you also have the correct Forge version up and running. You're good to go!\n");
            }
        } else {
            println!("Unfortunately, we can only help you to setup your installation correctly when you use the default way.\n");
        }
    } else {
        println!("Failed to retrieve the latest version. Please try again.\n");
    }

    println!("Press Enter to exit...");
    let mut _exit = String::new();
    io::stdin().read_line(&mut _exit).unwrap();
}

fn get_latest_mod_version(url: &str) -> Option<String> {
    match get(url) {
        Ok(response) => match response.text() {
            Ok(json) => {
                let parsed: Value = serde_json::from_str(&json).ok()?;
                parsed["file"].as_str().map(String::from)
            }
            Err(_) => None,
        },
        Err(_) => None,
    }
}

fn extract_version(file_name: &str) -> String {
    if let Some(start) = file_name.find("_v") {
        if let Some(end) = file_name.find(".jar") {
            return file_name[start + 1..end].to_string();
        }
    }
    "Unknown".to_string()
}

fn wait_for_download(file_path: &str) {
    let path = Path::new(file_path);
    
    // Warte, bis die Datei existiert
    while !path.exists() {
        thread::sleep(Duration::from_secs(2));
    }

    let mut last_size = 0;
    
    loop {
        if let Ok(meta) = metadata(file_path) {
            let size = meta.len();
            if size > 0 && size == last_size {
                break;
            }
            last_size = size;
        }
        thread::sleep(Duration::from_secs(2));
    }
}