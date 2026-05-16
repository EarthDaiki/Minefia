mod ai;
mod tools;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ai::ollama::ask_ollama])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}