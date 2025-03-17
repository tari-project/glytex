use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use log::warn;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct GpuStatus {
    pub recommended_grid_size: u32,
    pub recommended_block_size: u32,
    pub max_grid_size: u32,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct GpuSettings {
    pub is_excluded: bool,
    pub is_available: bool,
}

impl Default for GpuSettings {
    fn default() -> Self {
        Self {
            is_excluded: false,
            is_available: true,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct GpuDevice {
    pub device_name: String,
    pub device_index: u32,
    pub status: GpuStatus,
    pub settings: GpuSettings,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct GpuStatusFile {
    pub gpu_devices: Vec<GpuDevice>,
}

impl Default for GpuStatusFile {
    fn default() -> Self {
        Self {
            gpu_devices: Vec::new(),
        }
    }
}

impl GpuStatusFile {
    pub fn new(gpu_devices: Vec<GpuDevice>, file_path: &PathBuf) -> Self {
        let resolved_gpu_file_content = Self::resolve_settings_for_detected_devices(gpu_devices, file_path);

        Self {
            gpu_devices: resolved_gpu_file_content,
        }
    }

    pub fn load<T: AsRef<Path>>(path: T) -> Result<Self, anyhow::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    pub fn save<T: AsRef<Path>>(&self, path: T) -> Result<(), anyhow::Error> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    fn resolve_settings_for_detected_devices(gpu_devices: Vec<GpuDevice>, file_path: &PathBuf) -> Vec<GpuDevice> {
        match Self::load(file_path) {
            Ok(file) => gpu_devices
                .into_iter()
                .map(|device| {
                    let device_index = device.device_index.clone();
                    match file.gpu_devices.iter().find(|d| d.device_index == device_index) {
                        Some(existing_device) => {
                            let mut resolved_device = device.clone();
                            resolved_device.settings = existing_device.settings.clone();
                            resolved_device
                        },
                        None => device,
                    }
                })
                .collect(),
            Err(e) => {
                warn!("Could not load GPU status file: {}. Using detected devices", e);
                gpu_devices
            },
        }
    }
}
