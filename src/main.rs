#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(dead_code, unused_imports)]

use eframe::egui;
use egui::{
    CentralPanel,
    Frame
};
use std::path::{
    PathBuf,
    Path
};
use dirs::home_dir;
use system_uri;
use anyhow::{
    Result,
    Context
};
use std::fs::{create_dir_all, write, read_to_string};
use std::io::{Cursor, Write};
use lazy_static::lazy_static;
use rand::RngCore;

lazy_static! {
    static ref IMAGE_NUM: usize = (rand::thread_rng().next_u32() % 4) as usize;  //egui::ImageSource<'static> = 
        
}

fn install_uri_handler() -> Result<()> {
    let exec: String = std::env::current_exe()?.to_str().context("failed string conversion")?.to_owned();
    let uri_scheme_app = system_uri::App::new(
        "net.aemi.installer".to_owned(),
        "o7Moon".to_owned(),
        "Aemi".to_owned(),
        exec,
        None,
    );
    system_uri::install(&uri_scheme_app, &["aemi".to_owned()]).context("failed to install uri scheme")?;
    Ok(())
}

fn set_install_path(path: String) -> Result<()> {
    let config_dir = dirs::config_dir().context("failed to get config dir")?.join("aemi");
    create_dir_all(&config_dir)?;
    write(config_dir.join("install_path.txt"), path)?;
    Ok(())
}

fn get_install_path() -> Result<String> {
    Ok(read_to_string(
        dirs::config_dir().context("failed to get config dir")?
        .join("aemi")
        .join("install_path.txt")
    ).unwrap_or(|| -> Result<String> {
        Ok(
        if cfg!(target_os = "windows") {
            r"C:\Program Files (x86)\Steam\steamapps\common\A Difficult Game About Climbing".to_owned()
        } else {
            home_dir().context("failed to get the home dir")?
            .join(r".steam/steam/steamapps/common/A Difficult Game About Climbing")
            .to_str().context("failed to convert to string")?.to_owned()
        })
    }()?))
}

fn update_main_menu(ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let theme = egui::Visuals {
        widgets: egui::style::Widgets {
            inactive: egui::style::WidgetVisuals {
                bg_fill: egui::Color32::from_black_alpha(32),
                bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_white_alpha(8)),
                fg_stroke: egui::Stroke::new(2.0, egui::Color32::WHITE),
                weak_bg_fill: egui::Color32::from_black_alpha(128),
                rounding: egui::Rounding::same(4.0),
                expansion: 1.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: egui::Color32::from_black_alpha(64),
                bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_white_alpha(0)),
                fg_stroke: egui::Stroke::new(2.0, egui::Color32::WHITE),
                weak_bg_fill: egui::Color32::from_black_alpha(64),
                rounding: egui::Rounding::same(4.0),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: egui::Color32::from_black_alpha(0),
                bg_stroke: egui::Stroke::new(0.0, egui::Color32::from_white_alpha(4)),
                fg_stroke: egui::Stroke::new(0.0, egui::Color32::WHITE),
                weak_bg_fill: egui::Color32::from_black_alpha(32),
                rounding: egui::Rounding::same(4.0),
                expansion: 1.0,
            },
            ..Default::default()
        },
        ..Default::default()
    };
    ctx.set_visuals(theme);
    ctx.style_mut(|style| {
        style.override_text_style = Some(egui::TextStyle::Heading);
        style.spacing = egui::style::Spacing {
            item_spacing: egui::vec2(10.0, 10.0),
            ..Default::default()
        };
    });
    egui_extras::install_image_loaders(ctx);
    CentralPanel::default().frame(Frame::none().stroke(egui::Stroke::new(4.0, egui::Color32::from_rgb(36,61,115))).fill(egui::Color32::from_rgb(29,51,99))).show(ctx, |ui|{
        const IMGS: [egui::ImageSource; 4] = [
            egui::include_image!("../assets/background_00.png"), 
            egui::include_image!("../assets/background_01.png"), 
            egui::include_image!("../assets/background_02.png"), 
            egui::include_image!("../assets/background_03.png")
        ];
        let img = IMGS[*IMAGE_NUM].clone();
        egui::Image::new(
            img
        ).paint_at(ui, ui.ctx().screen_rect());
        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui|{
        ui.add_space(13.0);
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(13.0);
            if ui.button("exit").clicked() {
                std::process::exit(0);
            }
            if ui.button("install BepInEx").clicked() {
                let res = install_bepinex();
                if res.is_err() {
                    alert_error(res.expect_err("uh"));
                } else if res.unwrap() {
                    let _ = native_dialog::MessageDialog::new()
                        .set_type(native_dialog::MessageType::Info)
                        .set_text("BepInEx sucessfully installed! you may need to run the game once and then close it for mods to start working.")
                        .show_alert();
                }
            }
            if ui.button("uninstall BepInEx").clicked() {
                let res = uninstall_bepinex();
                if res.is_err() {
                    alert_error(res.expect_err("uh"));
                }
            }
            if ui.button("install .dll mods").clicked() {
                let res = copy_dll_from_file_dialog();
                if res.is_err() {
                    alert_error(res.expect_err("uh"));
                }
            }
            if ui.button("set game path").clicked() {
                let res = change_install_path();
                if res.is_err() {
                    alert_error(res.expect_err("uh"));
                }
            }
        })
        });
    });
}

fn alert_error(err: anyhow::Error) {
    let _ = native_dialog::MessageDialog::new()
        .set_text(("Encountered an error: ".to_owned() + err.to_string().as_str()).as_str())
        .set_type(native_dialog::MessageType::Error)
        .show_alert(); 
}

const BEPINEX_URL: &str = "https://github.com/BepInEx/BepInEx/releases/download/v5.4.22/BepInEx_x64_5.4.22.0.zip";

fn install_bepinex() -> Result<bool> {
    let installpath = PathBuf::from(get_install_path()?);
    if installpath.join("BepInEx").exists() {
        let continue_ = native_dialog::MessageDialog::new()
            .set_text("BepInEx folder already exists. do you want to delete it?")
            .set_type(native_dialog::MessageType::Warning)
            .show_confirm()?;
        if !continue_ {
            return Ok(false);
        }
        std::fs::remove_dir_all(installpath.join("BepInEx"))?;
    }
    let client = reqwest::blocking::Client::new();
    let mut response = client.get(BEPINEX_URL)
        .send().context("failed to fetch bepinex. is github down?")?;
    let mut content: Vec<u8> = Vec::new();
    response.copy_to(&mut content).context("failed copying response body")?;
    let mut archive = zip::ZipArchive::new(Cursor::new(content)).context("failed to load zip")?;
    archive.extract(installpath).context("failed to extract zip")?;
    Ok(true)
}

fn uninstall_bepinex() -> Result<()> {
    let installpath = PathBuf::from(get_install_path()?);
    if !installpath.join("BepInEx").exists() {
        native_dialog::MessageDialog::new()
            .set_type(native_dialog::MessageType::Error)
            .set_text("BepInEx doesnt exist at the game path! nothing to uninstall.")
            .show_alert()?;
        return Ok(());
    }
    let continue_ = native_dialog::MessageDialog::new()
        .set_text("are you sure you want to uninstall BepInEx?")
        .set_type(native_dialog::MessageType::Warning)
        .show_confirm()?;
    if !continue_ {
        return Ok(());
    }
    std::fs::remove_dir_all(installpath.join("BepInEx"))?;
    std::fs::remove_file(installpath.join("winhttp.dll"))?;
    Ok(())
}

fn copy_dll_from_file_dialog() -> Result<()> {
    let installpath = PathBuf::from(get_install_path()?);
    if !installpath.join("BepInEx").exists() {
        native_dialog::MessageDialog::new()
            .set_type(native_dialog::MessageType::Error)
            .set_text("BepInEx doesnt exist at the game path! install BepInEx first.")
            .show_alert()?;
        return Ok(());
    }
    let files = native_dialog::FileDialog::new()
        .set_title("select .dll mods to copy to bepinex's plugins folder")
        .add_filter("dll mod", &["dll"])
        .show_open_multiple_file()?;
    if !installpath.join("BepInEx/plugins").exists() {
        std::fs::create_dir_all(installpath.join("BepInEx/plugins"))?;
    }
    let plugins_dir = installpath.join("BepInEx/plugins");
    for file in files {
        std::fs::copy(file.clone(), plugins_dir.join(file.clone().file_name().context("file is actually a directory i think?")?))?;
    }
    Ok(())
}

fn change_install_path() -> Result<()> {
    let installpath = get_install_path()?;
    let dir = native_dialog::FileDialog::new()
        .set_location(&installpath)
        .show_open_single_dir()?.unwrap_or(installpath.into());
    set_install_path(dir.to_str().context("failed string conversion")?.to_owned())?;
    Ok(())
}


fn download_and_install_mod(url: String) -> Result<()> {
    let installpath = PathBuf::from(get_install_path()?);
    let filename = url.split("/").last().context("invalid url")?;
    let client = reqwest::blocking::Client::new();
    let mut response = client.get(url.clone())
        .send().expect("failed to download mod");
    let mut content: Vec<u8> = Vec::new();
    response.copy_to(&mut content)?;
    
    let file_extension = filename.rsplit_once(".").context("no file extension?")?.1;
    match file_extension {
        "dll" => {
            let path: PathBuf = [installpath,"BepInEx".into(),"plugins".into(),filename.into()].iter().collect();
            let mut file = std::fs::File::create(path).context("failed to create mod file")?;
            file.write_all(&content)?;
            Ok(())
        },
        "zip" => {
            let path: PathBuf = [installpath,"BepInEx".into(),"plugins".into()].iter().collect();
            let mut archive = zip::ZipArchive::new(Cursor::new(content)).context("failed to create zip file")?;
            archive.extract(path)?;
            Ok(())
        },
        _ => {
            None.context("url must end with .dll or .zip")?
        }
    }
}

fn main() -> eframe::Result<()> {
    let _ = install_uri_handler();
    let arg = std::env::args().nth(1);
    if let Some(arg) = arg {
        match arg {
            arg if arg.starts_with("aemi://installmod/") => {
                let url = arg.split_once("//installmod/").unwrap().1;
                let confirm = native_dialog::MessageDialog::new()
                    .set_text(&("are you sure you want to install \"".to_owned() + &url + "\"?"))
                    .set_type(native_dialog::MessageType::Warning)
                    .show_confirm().unwrap();
                if !confirm {
                    return Ok(());
                }
                let res = download_and_install_mod(url.to_string());
                if res.is_err() {
                    alert_error(res.expect_err("uh"));
                }
                return Ok(())
            },
            _ => {}
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder {
            title: Some("aemi".to_owned()),
            inner_size: Some(egui::vec2(512.0, 288.0)),
            decorations: Some(false),
            resizable: Some(false),
            ..Default::default()  
        },
        ..Default::default()
    };
    eframe::run_simple_native("aemi", options, update_main_menu)
}
