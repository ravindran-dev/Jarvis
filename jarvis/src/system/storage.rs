use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
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

    current_path: Arc<Mutex<Option<PathBuf>>>,

    // Cache subdirectory listings and sizes per parent path to avoid recomputation
    subdir_cache: Arc<Mutex<HashMap<String, Vec<DirectoryItem>>>>,
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
            current_path: Arc::new(Mutex::new(None)),
            subdir_cache: Arc::new(Mutex::new(HashMap::new())),
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
            sorted_results.sort_by_key(|a| std::cmp::Reverse(a.size));

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

    pub fn get_current_path(&self) -> Option<PathBuf> {
        self.current_path.lock().unwrap().clone()
    }

    pub fn set_current_path(&mut self, path: Option<PathBuf>) {
        *self.current_path.lock().unwrap() = path;
    }

    pub fn get_subdirectories(&self, parent_path: &str) -> Vec<DirectoryItem> {
        let path = Path::new(parent_path);
        if !path.is_dir() {
            return Vec::new();
        }

        // If we have cached results, return them sorted by size
        if let Some(cached) = self.subdir_cache.lock().unwrap().get(parent_path).cloned() {
            let mut cached_sorted = cached.clone();
            cached_sorted.sort_by_key(|a| std::cmp::Reverse(a.size));
            return cached_sorted;
        }

        // Quick listing without deep size computation (fast path)
        let mut initial: Vec<DirectoryItem> = Vec::new();
        let mut child_paths: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let child = entry.path();
                if child.is_dir() {
                    child_paths.push(child.clone());
                    if let Ok(p) = child.canonicalize() {
                        initial.push(DirectoryItem {
                            path: p.to_string_lossy().to_string(),
                            size: 0,
                            file_count: 0,
                        });
                    }
                }
            }
        }

        // Insert the initial placeholder list into cache so subsequent renders use it
        {
            let mut cache = self.subdir_cache.lock().unwrap();
            cache.insert(parent_path.to_string(), initial.clone());
        }

        // Spawn background computation to calculate sizes and update cache
        let cache = Arc::clone(&self.subdir_cache);
        let parent = parent_path.to_string();
        thread::spawn(move || {
            let computed: Vec<DirectoryItem> = child_paths
                .par_iter()
                .filter_map(|child_path| {
                    let (size, file_count) = Self::calculate_directory_size(child_path);
                    if let Ok(p) = child_path.canonicalize() {
                        Some(DirectoryItem {
                            path: p.to_string_lossy().to_string(),
                            size,
                            file_count,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            let mut cache_lock = cache.lock().unwrap();
            cache_lock.insert(parent, computed);
        });

        // Return the initial fast list (sizes 0); UI will refresh when cache fills
        initial
    }
}
