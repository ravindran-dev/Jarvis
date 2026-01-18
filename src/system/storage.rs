use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;


#[derive(Debug, Clone)]
pub struct DirectoryItem {
    pub path: String,
    pub size: u64,
    pub file_count: usize,
}


pub struct StorageAnalyzer {

    results: Arc<Mutex<Vec<DirectoryItem>>>,

    scanning: Arc<Mutex<bool>>,
 
    scan_paths: Vec<PathBuf>,

    min_threshold_bytes: u64,
}

impl StorageAnalyzer {
  
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));

        let scan_paths = vec![
            home_dir.clone(),
            PathBuf::from("/var/cache"),
            PathBuf::from("/var/log"),
            PathBuf::from("/var/lib/docker"),
            PathBuf::from("/tmp"),
        ];

        let analyzer = Self {
            results: Arc::new(Mutex::new(Vec::new())),
            scanning: Arc::new(Mutex::new(false)),
            scan_paths,
            min_threshold_bytes: 1024 * 1024,
        };

        analyzer.start_scan()?;

        Ok(analyzer)
    }

    
    pub fn is_scanning(&self) -> bool {
        *self.scanning.lock().unwrap()
    }


    pub fn get_results(&self) -> Vec<DirectoryItem> {
        self.results.lock().unwrap().clone()
    }

    pub fn get_results_count(&self) -> usize {
        self.results.lock().unwrap().len()
    }

    pub fn get_selected_item(&self, index: usize) -> Option<DirectoryItem> {
        self.results.lock().unwrap().get(index).cloned()
    }

   
    pub fn start_scan(&self) -> Result<()> {
        if self.is_scanning() {
            return Ok(());
        }

        let results = Arc::clone(&self.results);
        let scanning = Arc::clone(&self.scanning);
        let paths = self.scan_paths.clone();
        let threshold = self.min_threshold_bytes;

        *scanning.lock().unwrap() = true;

        thread::spawn(move || {
            let scan_results = Self::scan_directories(&paths, threshold);

            let mut sorted_results = scan_results;
            sorted_results.sort_by(|a, b| b.size.cmp(&a.size));

            sorted_results.truncate(50);

            *results.lock().unwrap() = sorted_results;

            *scanning.lock().unwrap() = false;
        });

        Ok(())
    }

 
    fn scan_directories(paths: &[PathBuf], min_threshold_bytes: u64) -> Vec<DirectoryItem> {
        paths
            .par_iter()
            .filter(|path| path.exists())
            .flat_map(|path| Self::scan_directory(path, min_threshold_bytes))
            .collect()
    }

   
    fn scan_directory(root: &Path, min_threshold_bytes: u64) -> Vec<DirectoryItem> {
        let mut directory_sizes: Vec<DirectoryItem> = Vec::new();

        if let Ok(entries) = std::fs::read_dir(root) {
            let children: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.path())
                .collect();

            let results: Vec<DirectoryItem> = children
                .par_iter()
                .filter_map(|child_path| {
                    let size_info = Self::calculate_directory_size(child_path);
                    if size_info.0 >= min_threshold_bytes {
                        Some(DirectoryItem {
                            path: child_path.to_string_lossy().to_string(),
                            size: size_info.0,
                            file_count: size_info.1,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            directory_sizes.extend(results);
        }

        directory_sizes
    }


    fn calculate_directory_size(path: &Path) -> (u64, usize) {
        let mut total_size = 0u64;
        let mut file_count = 0usize;

        for entry in WalkDir::new(path)
            .follow_links(false)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }

        (total_size, file_count)
    }

  
    pub fn set_min_threshold_bytes(&mut self, bytes: u64) {
        self.min_threshold_bytes = bytes;
    }
}
