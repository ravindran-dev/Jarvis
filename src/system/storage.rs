use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;

/// Represents a directory with its size and file count
#[derive(Debug, Clone)]
pub struct DirectoryItem {
    pub path: String,
    pub size: u64,
    pub file_count: usize,
}

/// Storage analyzer for scanning directories
pub struct StorageAnalyzer {
    /// Current scan results
    results: Arc<Mutex<Vec<DirectoryItem>>>,
    /// Whether a scan is currently in progress
    scanning: Arc<Mutex<bool>>,
    /// Paths to scan
    scan_paths: Vec<PathBuf>,
    /// Minimum size threshold to include a directory (bytes)
    min_threshold_bytes: u64,
}

impl StorageAnalyzer {
    /// Create a new StorageAnalyzer
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
            min_threshold_bytes: 1024 * 1024, // 1 MB default
        };

        // Start initial scan
        analyzer.start_scan()?;

        Ok(analyzer)
    }

    /// Check if a scan is currently in progress
    pub fn is_scanning(&self) -> bool {
        *self.scanning.lock().unwrap()
    }

    /// Get the current scan results
    pub fn get_results(&self) -> Vec<DirectoryItem> {
        self.results.lock().unwrap().clone()
    }

    /// Get the number of results
    pub fn get_results_count(&self) -> usize {
        self.results.lock().unwrap().len()
    }

    /// Start a background scan of configured directories
    pub fn start_scan(&self) -> Result<()> {
        // Check if already scanning
        if self.is_scanning() {
            return Ok(());
        }

        let results = Arc::clone(&self.results);
        let scanning = Arc::clone(&self.scanning);
        let paths = self.scan_paths.clone();
        let threshold = self.min_threshold_bytes;

        // Mark as scanning
        *scanning.lock().unwrap() = true;

        // Spawn background thread for scanning
        thread::spawn(move || {
            let scan_results = Self::scan_directories(&paths, threshold);

            // Sort by size descending
            let mut sorted_results = scan_results;
            sorted_results.sort_by(|a, b| b.size.cmp(&a.size));

            // Take top 50 largest
            sorted_results.truncate(50);

            // Update results
            *results.lock().unwrap() = sorted_results;

            // Mark as complete
            *scanning.lock().unwrap() = false;
        });

        Ok(())
    }

    /// Scan multiple directories in parallel
    fn scan_directories(paths: &[PathBuf], min_threshold_bytes: u64) -> Vec<DirectoryItem> {
        paths
            .par_iter()
            .filter(|path| path.exists())
            .flat_map(|path| Self::scan_directory(path, min_threshold_bytes))
            .collect()
    }

    /// Scan a single directory and return items
    fn scan_directory(root: &Path, min_threshold_bytes: u64) -> Vec<DirectoryItem> {
        let mut directory_sizes: Vec<DirectoryItem> = Vec::new();

        // First level: direct children of root
        if let Ok(entries) = std::fs::read_dir(root) {
            let children: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.path())
                .collect();

            // Process children in parallel
            let results: Vec<DirectoryItem> = children
                .par_iter()
                .filter_map(|child_path| {
                    let size_info = Self::calculate_directory_size(child_path);
                    if size_info.0 >= min_threshold_bytes {
                        // Only include dirs > 1MB
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

    /// Calculate total size of a directory recursively
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

    /// Update the minimum size threshold (in bytes)
    pub fn set_min_threshold_bytes(&mut self, bytes: u64) {
        self.min_threshold_bytes = bytes;
    }
}
