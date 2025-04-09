// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri_plugin_dialog::FileDialogBuilder;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub username: String,
    pub host: String,
    pub port: u16,
    pub auth_type: String,
    pub auth_info: String,
    pub group_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    groups: Vec<Group>,
    servers: Vec<Server>,
}

fn get_data_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "ssh-book", "ssh-book")
        .expect("Failed to get project directories");
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir).expect("Failed to create data directory");
    data_dir.join("data.json")
}

fn load_data() -> Result<Data, Error> {
    let path = get_data_path();
    println!("Data path: {:?}", path);
    if !path.exists() {
        println!("Creating new data file");
        return Ok(Data {
            groups: Vec::new(),
            servers: Vec::new(),
        });
    }
    let data = fs::read_to_string(&path)?;
    println!("Loaded data: {}", data);
    Ok(serde_json::from_str(&data)?)
}

fn save_data(data: &Data) -> Result<(), Error> {
    let path = get_data_path();
    let json = serde_json::to_string_pretty(data)?;
    println!("Saving data: {}", json);
    fs::write(path, json)?;
    Ok(())
}

#[tauri::command]
fn get_groups() -> Result<Vec<Group>, String> {
    println!("Getting groups");
    load_data()
        .map(|data| {
            println!("Found {} groups", data.groups.len());
            data.groups
        })
        .map_err(|e| {
            println!("Failed to get groups: {}", e);
            e.to_string()
        })
}

#[tauri::command]
fn get_servers() -> Result<Vec<Server>, String> {
    println!("Getting servers");
    load_data()
        .map(|data| {
            println!("Found {} servers", data.servers.len());
            data.servers
        })
        .map_err(|e| {
            println!("Failed to get servers: {}", e);
            e.to_string()
        })
}

#[tauri::command]
fn add_group(name: String) -> Result<(), String> {
    let mut data = load_data().map_err(|e| e.to_string())?;
    let group = Group {
        id: Uuid::new_v4().to_string(),
        name,
    };
    data.groups.push(group);
    save_data(&data).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_server(server: Server) -> Result<(), String> {
    println!("Adding server: {:?}", server);
    let mut data = load_data().map_err(|e| {
        println!("Failed to load data: {}", e);
        e.to_string()
    })?;

    // 验证分组是否存在
    if !data.groups.iter().any(|g| g.id == server.group_id) {
        return Err(format!("Group with id {} not found", server.group_id));
    }

    data.servers.push(server);
    save_data(&data).map_err(|e| {
        println!("Failed to save data: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn delete_group(id: String) -> Result<(), String> {
    println!("Deleting group: {}", id);
    let mut data = load_data().map_err(|e| e.to_string())?;

    // 删除组
    if let Some(index) = data.groups.iter().position(|g| g.id == id) {
        data.groups.remove(index);
        println!("Group removed successfully");
    } else {
        println!("Group not found: {}", id);
        return Err("Group not found".to_string());
    }

    // 删除该组下的所有服务器
    let servers_count = data.servers.len();
    data.servers.retain(|s| s.group_id != id);
    println!(
        "Removed {} servers from group",
        servers_count - data.servers.len()
    );

    // 保存数据
    if let Err(e) = save_data(&data) {
        println!("Failed to save data: {}", e);
        return Err(format!("Failed to save data: {}", e));
    }

    println!("Data saved successfully");
    Ok(())
}

#[tauri::command]
async fn delete_server(id: String) -> Result<(), String> {
    println!("Deleting server: {}", id);
    let mut data = load_data().map_err(|e| e.to_string())?;

    if let Some(index) = data.servers.iter().position(|s| s.id == id) {
        data.servers.remove(index);
        println!("Server removed successfully");
    } else {
        println!("Server not found: {}", id);
        return Err("Server not found".to_string());
    }

    // 保存数据
    if let Err(e) = save_data(&data) {
        println!("Failed to save data: {}", e);
        return Err(format!("Failed to save data: {}", e));
    }

    println!("Data saved successfully");
    Ok(())
}

#[tauri::command]
fn connect_server(server: Server) -> Result<(), String> {
    let mut command = format!("ssh {}@{} -p {}", server.username, server.host, server.port);

    match server.auth_type.as_str() {
        "password" => {
            // 密码登录需要用户交互输入密码
            command = format!("{} -o PreferredAuthentications=password", command);
        }
        "key" => {
            command = format!("{} -i {}", command, server.auth_info);
        }
        _ => return Err("Invalid authentication type".to_string()),
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("osascript")
            .arg("-e")
            .arg(format!(
                "tell application \"Terminal\" to do script \"{}\"",
                command
            ))
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn update_group(group: Group) -> Result<(), String> {
    let mut data = load_data().map_err(|e| e.to_string())?;
    if let Some(index) = data.groups.iter().position(|g| g.id == group.id) {
        data.groups[index] = group;
        save_data(&data).map_err(|e| e.to_string())
    } else {
        Err("Group not found".to_string())
    }
}

#[tauri::command]
fn update_server(server: Server) -> Result<(), String> {
    let mut data = load_data().map_err(|e| e.to_string())?;
    if let Some(index) = data.servers.iter().position(|s| s.id == server.id) {
        data.servers[index] = server;
        save_data(&data).map_err(|e| e.to_string())
    } else {
        Err("Server not found".to_string())
    }
}

#[tauri::command]
fn export_data() {
    // FileDialogBuilder::new()
    //     .set_file_name("default.txt")
    //     .save_file(|path| {
    //         if let Some(path) = path {
    //             println!("用户保存文件到: {:?}", path);
    //             let data = load_data().map_err(|e| e.to_string())?;
    //             let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    //             std::fs::write(&path, json).map_err(|e| e.to_string())?;
    //         } else {
    //             println!("用户取消了保存操作");
    //         }
    //     });
}

#[tauri::command]
fn import_data() {
    // 获取用户选择的文件
    // tauri::api::dialog::FileDialogBuilder::new()
    //     .add_filter("Text Files", &["txt"])
    //     .pick_file(|file_path| {
    //         if let Some(path) = file_path {
    //             println!("用户选择的文件路径: {:?}", path);
    //             // 读取文件
    //             let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    //             let data: Data = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    //             // 保存数据
    //             save_data(&data).map_err(|e| e.to_string())?;
    //         } else {
    //             println!("用户取消了文件选择");
    //         }
    //     });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_groups,
            get_servers,
            add_group,
            add_server,
            delete_group,
            delete_server,
            connect_server,
            update_group,
            update_server,
            export_data,
            import_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
