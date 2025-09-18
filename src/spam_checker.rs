use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize)]
pub struct SpamLabel {
    pub provider: u64,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    pub label_type: String,
    pub label_value: u8,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TargetType {
    pub target: String,
    pub fid: u64,
}

pub struct SpamChecker {
    labels: HashMap<u64, SpamLabel>,
}

impl SpamChecker {
    /// Load spam labels from JSONL file
    pub fn load_from_file(file_path: &str) -> Result<Self> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut labels = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<SpamLabel>(&line) {
                Ok(label) => {
                    // Only process spam labels
                    if label.label_type == "spam" {
                        labels.insert(label.target_type.fid, label);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse line: {} - Error: {}", line, e);
                }
            }
        }

        Ok(SpamChecker { labels })
    }

    /// Check if a FID is marked as spam
    pub fn is_spam(&self, fid: u64) -> Option<bool> {
        self.labels.get(&fid).map(|label| label.label_value == 0)
    }

    /// Get spam label for a FID
    pub fn get_label(&self, fid: u64) -> Option<&SpamLabel> {
        self.labels.get(&fid)
    }

    /// Get all spam FIDs
    pub fn get_spam_fids(&self) -> Vec<u64> {
        self.labels
            .iter()
            .filter(|(_, label)| label.label_value == 0)
            .map(|(fid, _)| *fid)
            .collect()
    }

    /// Get all non-spam FIDs
    pub fn get_non_spam_fids(&self) -> Vec<u64> {
        self.labels
            .iter()
            .filter(|(_, label)| label.label_value == 2)
            .map(|(fid, _)| *fid)
            .collect()
    }

    /// Get statistics
    pub fn get_stats(&self) -> (usize, usize, usize) {
        let total = self.labels.len();
        let spam_count = self.labels.values().filter(|l| l.label_value == 0).count();
        let non_spam_count = self.labels.values().filter(|l| l.label_value == 2).count();
        (total, spam_count, non_spam_count)
    }

    /// Check multiple FIDs at once
    pub fn check_multiple(&self, fids: &[u64]) -> HashMap<u64, Option<bool>> {
        fids.iter().map(|&fid| (fid, self.is_spam(fid))).collect()
    }

    /// Get the oldest timestamp from all labels
    pub fn get_oldest_timestamp(&self) -> Option<u64> {
        self.labels.values().map(|label| label.timestamp).min()
    }

    /// Get the newest timestamp from all labels
    pub fn get_newest_timestamp(&self) -> Option<u64> {
        self.labels.values().map(|label| label.timestamp).max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spam_checker() {
        // This would require a test JSONL file
        // let checker = SpamChecker::load_from_file("test_spam.jsonl").unwrap();
        // assert_eq!(checker.is_spam(12345), Some(true));
    }
}
