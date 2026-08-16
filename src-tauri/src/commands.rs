#[tauri::command]
pub async fn start_recording() -> Result<(), String> {
    println!("Recording started!");
    // Initialize capture and muxer here
    Ok(())
}

#[tauri::command]
pub async fn stop_recording() -> Result<(), String> {
    println!("Recording stopped!");
    Ok(())
}
