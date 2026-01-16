use anyhow::Result;
use log::debug;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
use walkdir::WalkDir;

/// Progress information for storage scanning
#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub scanned_items: usize,
    pub directories_analyzed: usize,
    pub total_size: u64,
    pub is_complete: bool,
}

/// Represents a directory entry with size information
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub path: PathBuf,
    pub size: u64,
    pub file_count: usize,
}

/// Optimized storage scanner using bounded thread pool
pub struct StorageScanner {
    thread_pool: ThreadPool,
    progress: Arc<Mutex<ScanProgress>>,
}

impl StorageScanner {
    /// Create a new scanner with bounded thread pool (4 threads by default)
    pub fn new() -> Self {
        Self::with_threads(4)
    }

    /// Create scanner with specified number of threads
    pub fn with_threads(num_threads: usize) -> Self {
        let thread_pool = ThreadPool::new(num_threads);
        let progress = Arc::new(Mutex::new(ScanProgress {
            scanned_items: 0,
            directories_analyzed: 0,
            total_size: 0,
            is_complete: false,
        }));

        Self {
            thread_pool,
            progress,
        }
    }

    /// Scan directory and return top directories by size
    pub fn scan_directory(&self, path: &Path, top_n: usize) -> Result<Vec<DirectoryEntry>> {
        debug!("Starting directory scan of: {:?}", path);

        if let Ok(mut p) = self.progress.lock() {
            p.scanned_items = 0;
            p.directories_analyzed = 0;
            p.total_size = 0;
            p.is_complete = false;
        }

        let mut entries = Vec::new();

        for entry in WalkDir::new(path)
            .max_depth(3)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();

            if let Ok(mut p) = self.progress.lock() {
                p.scanned_items += 1;
            }

            if path.is_dir() {
                if let Ok(_metadata) = path.metadata() {
                    let size = Self::get_dir_size(path)?;
                    let file_count = Self::count_files(path)?;

                    entries.push(DirectoryEntry {
                        path: path.to_path_buf(),
                        size,
                        file_count,
                    });

                    if let Ok(mut p) = self.progress.lock() {
                        p.directories_analyzed += 1;
                        p.total_size += size;
                    }
                }
            }
        }

        entries.sort_by(|a, b| b.size.cmp(&a.size));
        entries.truncate(top_n);

        if let Ok(mut p) = self.progress.lock() {
            p.is_complete = true;
        }

        debug!("Scan complete: {} directories found", entries.len());
        Ok(entries)
    }

    /// Get directory size efficiently
    fn get_dir_size(path: &Path) -> Result<u64> {
        let mut size = 0u64;

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file())
        {
            if let Ok(metadata) = entry.metadata() {
                size += metadata.len();
            }
        }

        Ok(size)
    }

    /// Count files in directory
    fn count_files(path: &Path) -> Result<usize> {
        let count = WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file())
            .count();

        Ok(count)
    }

    /// Get current progress
    pub fn get_progress(&self) -> ScanProgress {
        self.progress.lock().unwrap().clone()
    }

    /// Wait for thread pool to finish
    pub fn join(self) {
        self.thread_pool.join();
    }
}

impl Default for StorageScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scanner_creation() {
        let scanner = StorageScanner::new();
        let progress = scanner.get_progress();
        assert_eq!(progress.scanned_items, 0);
        assert!(!progress.is_complete);
    }

    #[test]
    fn test_scanner_with_threads() {
        let scanner = StorageScanner::with_threads(2);
        let _progress = scanner.get_progress();
    }
}
