use serde::Serialize;
use tauri::State;
use crate::AppState;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub connected: bool,
    pub mount_point: Option<String>,
    pub confirmed: bool,
}

/// Get the current device connection status.
#[tauri::command]
#[specta::specta]
pub async fn get_device_status(
    state: State<'_, AppState>,
) -> Result<DeviceStatus, AppError> {
    let device = state.device.lock().map_err(|e| AppError::Device(e.to_string()))?;
    Ok(DeviceStatus {
        connected: device.mount_point.is_some(),
        mount_point: device.mount_point.as_ref().map(|p| p.to_string_lossy().to_string()),
        confirmed: device.confirmed,
    })
}

/// Confirm the detected OT volume for use.
/// Sets the device as confirmed and returns success.
#[tauri::command]
#[specta::specta]
pub async fn confirm_device(
    state: State<'_, AppState>,
    mount_point: String,
) -> Result<(), AppError> {
    let mut device = state.device.lock().map_err(|e| AppError::Device(e.to_string()))?;
    let path = std::path::PathBuf::from(&mount_point);

    // Validate the path exists and looks like an OT volume (T-01-14 mitigation)
    if !crate::device::is_ot_volume(&path) {
        return Err(AppError::Device(format!(
            "Path does not appear to be an Octatrack volume: {}",
            mount_point
        )));
    }

    device.mount_point = Some(path);
    device.confirmed = true;
    tracing::info!("Device confirmed: {}", mount_point);
    Ok(())
}

/// Dismiss/disconnect the device.
#[tauri::command]
#[specta::specta]
pub async fn dismiss_device(
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut device = state.device.lock().map_err(|e| AppError::Device(e.to_string()))?;
    device.mount_point = None;
    device.confirmed = false;
    tracing::info!("Device dismissed");
    Ok(())
}
