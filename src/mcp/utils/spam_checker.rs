//! Spam checker module for Farcaster FIDs
//!
//! This module loads spam labels from the Warpcast spam dataset
//! and provides fast lookup capabilities for spam detection.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;

use serde::Deserialize;
use tracing::debug;
use tracing::info;
use tracing::warn;

/// Spam label entry from the dataset
#[derive(Debug, Deserialize)]
struct SpamLabel {
    #[allow(dead_code)]
    provider: u64,
    #[serde(rename = "type")]
    type_info: LabelType,
    #[allow(dead_code)]
    label_type: String,
    label_value: u8,
    #[allow(dead_code)]
    timestamp: u64,
}

/// Label type with FID
#[derive(Debug, Deserialize)]
struct LabelType {
    #[allow(dead_code)]
    target: String,
    fid: u64,
}

/// Spam checker with loaded spam data
pub struct SpamChecker {
    spam_fids: HashMap<u64, bool>,
    total_labels: usize,
}

impl SpamChecker {
    /// Load spam labels from file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let spam_file = Self::spam_file_path()?;

        info!("Loading spam labels from: {}", spam_file.display());

        let file = File::open(&spam_file)?;
        let reader = BufReader::new(file);

        let mut spam_fids = HashMap::new();
        let mut total_labels = 0;
        let mut parse_errors = 0;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    warn!("Failed to read line: {}", e);
                    continue;
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<SpamLabel>(&line) {
                Ok(label) => {
                    total_labels += 1;

                    // label_value: 1 = spam, 0 = not spam
                    let is_spam = label.label_value == 1;
                    spam_fids.insert(label.type_info.fid, is_spam);
                }
                Err(e) => {
                    parse_errors += 1;
                    if parse_errors <= 5 {
                        debug!("Failed to parse line: {} - Error: {}", line, e);
                    }
                }
            }
        }

        info!(
            "Loaded {} spam labels ({} unique FIDs, {} parse errors)",
            total_labels,
            spam_fids.len(),
            parse_errors
        );

        Ok(Self {
            spam_fids,
            total_labels,
        })
    }

    /// Check if a FID is marked as spam
    pub fn is_spam(&self, fid: u64) -> bool {
        self.spam_fids.get(&fid).copied().unwrap_or(false)
    }

    /// Get spam status for a FID
    pub fn get_status(&self, fid: u64) -> SpamStatus {
        match self.spam_fids.get(&fid) {
            Some(true) => SpamStatus::Spam,
            Some(false) => SpamStatus::NotSpam,
            None => SpamStatus::Unknown,
        }
    }

    /// Get statistics about spam labels
    pub fn get_stats(&self) -> SpamStats {
        let spam_count = self.spam_fids.values().filter(|&&is_spam| is_spam).count();
        let non_spam_count = self.spam_fids.values().filter(|&&is_spam| !is_spam).count();

        SpamStats {
            total_labels: self.total_labels,
            unique_fids: self.spam_fids.len(),
            spam_count,
            non_spam_count,
            spam_percentage: if self.spam_fids.is_empty() {
                0.0
            } else {
                (spam_count as f64 / self.spam_fids.len() as f64) * 100.0
            },
        }
    }

    /// Get path to spam.jsonl file
    fn spam_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Try to find spam.jsonl in labels/labels/ directory
        let candidates = vec![
            PathBuf::from("labels/labels/spam.jsonl"),
            PathBuf::from("../labels/labels/spam.jsonl"),
            PathBuf::from("../../labels/labels/spam.jsonl"),
        ];

        for path in candidates {
            if path.exists() {
                return Ok(path);
            }
        }

        Err("Could not find spam.jsonl file. Please ensure labels submodule is initialized.".into())
    }
}

/// Spam status for a FID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpamStatus {
    Spam,
    NotSpam,
    Unknown,
}

/// Statistics about spam labels
#[derive(Debug, Clone)]
pub struct SpamStats {
    pub total_labels: usize,
    pub unique_fids: usize,
    pub spam_count: usize,
    pub non_spam_count: usize,
    pub spam_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spam_file_exists() {
        let result = SpamChecker::spam_file_path();
        assert!(result.is_ok(), "spam.jsonl file should be found");
    }

    #[test]
    fn test_load_spam_checker() {
        let checker = SpamChecker::load();
        if let Ok(checker) = checker {
            let stats = checker.get_stats();
            assert!(stats.total_labels > 0, "Should have loaded some labels");
            assert!(stats.unique_fids > 0, "Should have some unique FIDs");
        }
    }
}
