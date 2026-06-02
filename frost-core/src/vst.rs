pub mod host;

use std::path::PathBuf;
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VstPluginInfo {
    pub name: String,
    pub path: String,
    pub vendor: String,
    pub category: String,
    pub cid: Option<String>, // Class ID Node flawless
}

#[derive(Debug, serde::Deserialize)]
struct ModuleInfo {
    #[serde(rename = "Version")]
    _version: Option<String>,
    #[serde(rename = "Classes")]
    classes: Option<Vec<ClassInfo>>,
}

#[derive(Debug, serde::Deserialize)]
struct ClassInfo {
    name: Option<String>,
    category: Option<String>,
    vendor: Option<String>,
    cid: Option<String>, // TUID string mappings node Node-flawless Node flawless
}

/// Scans default OS directories for .vst3 files.
/// On Windows, this is typically `C:\Program Files\Common Files\VST3\`
pub fn scan_vst3_plugins() -> Vec<VstPluginInfo> {
    let mut plugins = Vec::new();
    
    #[cfg(windows)]
    let vst3_paths = vec![
        PathBuf::from(r"C:\Program Files\Common Files\VST3"),
        PathBuf::from(r"C:\Program Files (x86)\Common Files\VST3"),
    ];

    #[cfg(not(windows))]
    let vst3_paths: Vec<PathBuf> = vec![]; // Placeholder for macOS/Linux for now

    for base_path in vst3_paths {
        if !base_path.exists() {
            continue;
        }

        for entry in WalkDir::new(base_path)
            .into_iter()
            .filter_map(|e: Result<walkdir::DirEntry, walkdir::Error>| e.ok())
        {
            let path = entry.path();
            if path.is_file() || path.is_dir() {
                if let Some(ext) = path.extension() {
                    if ext == "vst3" {
                        let mut name = path.file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let mut vendor = "Unknown Vendor".to_string();
                        let mut category = "Effect".to_string();
                        let mut cid_val: Option<String> = None;

                        // Modern VST3.7+ bundles contain a moduleinfo.json
                        if path.is_dir() {
                            let mut info_path = path.to_path_buf();
                            info_path.push("moduleinfo.json");
                            if !info_path.exists() {
                                let mut alt_path = path.to_path_buf();
                                alt_path.push("Contents");
                                alt_path.push("moduleinfo.json");
                                if alt_path.exists() {
                                    info_path = alt_path;
                                }
                            }

                            if info_path.exists() {
                                if let Ok(content) = std::fs::read_to_string(&info_path) {
                                    if let Ok(module_info) = serde_json::from_str::<ModuleInfo>(&content) {
                                        if let Some(classes) = module_info.classes {
                                            if let Some(first_class) = classes.first() {
                                                if let Some(n) = &first_class.name { name = n.clone(); }
                                                if let Some(v) = &first_class.vendor { vendor = v.clone(); }
                                                if let Some(c) = &first_class.category { category = c.clone(); }
                                                if let Some(c_id) = &first_class.cid { cid_val = Some(c_id.clone()); }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        plugins.push(VstPluginInfo {
                            name,
                            path: path.to_string_lossy().to_string(),
                            vendor,
                            category,
                            cid: cid_val,
                        });
                    }
                }
            }
        }
    }

    plugins
}
