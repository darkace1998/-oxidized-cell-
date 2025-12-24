//! Trophy management
//!
//! Handles PS3 trophy unlocking and persistence

use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;

/// Trophy grade/rarity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrophyGrade {
    /// Bronze trophy
    Bronze,
    /// Silver trophy
    Silver,
    /// Gold trophy
    Gold,
    /// Platinum trophy
    Platinum,
}

/// Trophy type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrophyType {
    /// Standard trophy
    Standard,
    /// Hidden trophy (not shown until unlocked)
    Hidden,
}

/// Trophy information
#[derive(Debug, Clone)]
pub struct Trophy {
    /// Trophy ID
    pub id: u32,
    /// Trophy name
    pub name: String,
    /// Trophy description
    pub description: String,
    /// Trophy grade
    pub grade: TrophyGrade,
    /// Trophy type
    pub trophy_type: TrophyType,
    /// Whether the trophy is unlocked
    pub unlocked: bool,
    /// Unlock timestamp (if unlocked)
    pub unlock_time: Option<std::time::SystemTime>,
}

impl Trophy {
    /// Create a new trophy
    pub fn new(id: u32, name: String, description: String, grade: TrophyGrade) -> Self {
        Self {
            id,
            name,
            description,
            grade,
            trophy_type: TrophyType::Standard,
            unlocked: false,
            unlock_time: None,
        }
    }

    /// Unlock the trophy
    pub fn unlock(&mut self) {
        if !self.unlocked {
            self.unlocked = true;
            self.unlock_time = Some(std::time::SystemTime::now());
        }
    }
}

/// Trophy set for a game
#[derive(Debug, Clone)]
pub struct TrophySet {
    /// Game ID
    pub game_id: String,
    /// Game title
    pub title: String,
    /// Trophies in this set
    pub trophies: HashMap<u32, Trophy>,
    /// Trophy set icon path
    pub icon_path: Option<PathBuf>,
}

impl TrophySet {
    /// Create a new trophy set
    pub fn new(game_id: String, title: String) -> Self {
        Self {
            game_id,
            title,
            trophies: HashMap::new(),
            icon_path: None,
        }
    }

    /// Add a trophy to the set
    pub fn add_trophy(&mut self, trophy: Trophy) {
        self.trophies.insert(trophy.id, trophy);
    }

    /// Unlock a trophy
    pub fn unlock_trophy(&mut self, trophy_id: u32) -> bool {
        if let Some(trophy) = self.trophies.get_mut(&trophy_id) {
            if !trophy.unlocked {
                trophy.unlock();
                tracing::info!(
                    "Trophy unlocked: {} - {} ({})",
                    self.game_id,
                    trophy.name,
                    match trophy.grade {
                        TrophyGrade::Bronze => "Bronze",
                        TrophyGrade::Silver => "Silver",
                        TrophyGrade::Gold => "Gold",
                        TrophyGrade::Platinum => "Platinum",
                    }
                );
                return true;
            }
        }
        false
    }

    /// Check if a trophy is unlocked
    pub fn is_unlocked(&self, trophy_id: u32) -> bool {
        self.trophies
            .get(&trophy_id)
            .map(|t| t.unlocked)
            .unwrap_or(false)
    }

    /// Get total number of trophies
    pub fn total_trophies(&self) -> usize {
        self.trophies.len()
    }

    /// Get number of unlocked trophies
    pub fn unlocked_count(&self) -> usize {
        self.trophies.values().filter(|t| t.unlocked).count()
    }

    /// Get trophy progress percentage
    pub fn progress_percentage(&self) -> f32 {
        if self.trophies.is_empty() {
            return 0.0;
        }
        (self.unlocked_count() as f32 / self.total_trophies() as f32) * 100.0
    }

    /// Check if platinum trophy should be unlocked
    pub fn check_platinum(&mut self) -> bool {
        // Platinum is unlocked when all other trophies are unlocked
        let non_platinum_count = self
            .trophies
            .values()
            .filter(|t| t.grade != TrophyGrade::Platinum)
            .count();

        let unlocked_non_platinum = self
            .trophies
            .values()
            .filter(|t| t.grade != TrophyGrade::Platinum && t.unlocked)
            .count();

        if non_platinum_count > 0 && unlocked_non_platinum == non_platinum_count {
            // Find platinum trophy and unlock it
            if let Some((_, platinum)) = self
                .trophies
                .iter_mut()
                .find(|(_, t)| t.grade == TrophyGrade::Platinum)
            {
                if !platinum.unlocked {
                    platinum.unlock();
                    return true;
                }
            }
        }

        false
    }
}

/// Trophy manager
pub struct TrophyManager {
    /// Trophy sets by game ID
    sets: RwLock<HashMap<String, TrophySet>>,
    /// Base trophy directory
    base_path: PathBuf,
}

impl TrophyManager {
    /// Create a new trophy manager
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            sets: RwLock::new(HashMap::new()),
            base_path,
        }
    }

    /// Register a trophy set for a game
    pub fn register_set(&self, set: TrophySet) {
        let game_id = set.game_id.clone();
        self.sets.write().insert(game_id.clone(), set);
        tracing::info!("Registered trophy set for game: {}", game_id);
    }

    /// Unlock a trophy
    pub fn unlock_trophy(&self, game_id: &str, trophy_id: u32) -> Result<(), String> {
        let mut sets = self.sets.write();
        let set = sets
            .get_mut(game_id)
            .ok_or("Trophy set not found for game")?;

        if set.unlock_trophy(trophy_id) {
            // Check if platinum should be unlocked
            if set.check_platinum() {
                tracing::info!("Platinum trophy unlocked for game: {}", game_id);
            }

            // Save trophy data
            drop(sets);
            self.save_trophy_data(game_id)?;
            Ok(())
        } else {
            Err("Trophy already unlocked or not found".to_string())
        }
    }

    /// Get trophy set for a game
    pub fn get_set(&self, game_id: &str) -> Option<TrophySet> {
        self.sets.read().get(game_id).cloned()
    }

    /// List all trophy sets
    pub fn list_sets(&self) -> Vec<TrophySet> {
        self.sets.read().values().cloned().collect()
    }

    /// Save trophy data to disk
    fn save_trophy_data(&self, game_id: &str) -> Result<(), String> {
        let sets = self.sets.read();
        let set = sets.get(game_id).ok_or("Trophy set not found")?;

        let trophy_file = self.base_path.join(format!("{}_trophies.json", game_id));

        // Create directory if it doesn't exist
        if let Some(parent) = trophy_file.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create trophy directory: {}", e))?;
        }

        // Serialize trophy data (simplified)
        let mut data = String::new();
        data.push_str(&format!("game_id={}\n", set.game_id));
        data.push_str(&format!("title={}\n", set.title));
        
        for trophy in set.trophies.values() {
            if trophy.unlocked {
                data.push_str(&format!("unlocked={}\n", trophy.id));
            }
        }

        std::fs::write(&trophy_file, data)
            .map_err(|e| format!("Failed to save trophy data: {}", e))?;

        Ok(())
    }

    /// Load trophy data from disk
    pub fn load_trophy_data(&self, game_id: &str) -> Result<(), String> {
        let trophy_file = self.base_path.join(format!("{}_trophies.json", game_id));

        if !trophy_file.exists() {
            return Ok(()); // No saved data
        }

        let content = std::fs::read_to_string(&trophy_file)
            .map_err(|e| format!("Failed to read trophy data: {}", e))?;

        let mut unlocked_ids = Vec::new();
        for line in content.lines() {
            if let Some(id_str) = line.strip_prefix("unlocked=") {
                if let Ok(id) = id_str.parse::<u32>() {
                    unlocked_ids.push(id);
                }
            }
        }

        // Update trophy set
        let mut sets = self.sets.write();
        if let Some(set) = sets.get_mut(game_id) {
            for id in unlocked_ids {
                if let Some(trophy) = set.trophies.get_mut(&id) {
                    trophy.unlocked = true;
                }
            }
        }

        Ok(())
    }

    /// Initialize trophy directory
    pub fn init_trophy_directory(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.base_path)
            .map_err(|e| format!("Failed to create trophy directory: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trophy_creation() {
        let trophy = Trophy::new(
            1,
            "First Blood".to_string(),
            "Defeat your first enemy".to_string(),
            TrophyGrade::Bronze,
        );

        assert_eq!(trophy.id, 1);
        assert!(!trophy.unlocked);
    }

    #[test]
    fn test_trophy_unlock() {
        let mut trophy = Trophy::new(
            1,
            "Test Trophy".to_string(),
            "Test description".to_string(),
            TrophyGrade::Bronze,
        );

        assert!(!trophy.unlocked);
        trophy.unlock();
        assert!(trophy.unlocked);
        assert!(trophy.unlock_time.is_some());
    }

    #[test]
    fn test_trophy_set() {
        let mut set = TrophySet::new("BLES00000".to_string(), "Test Game".to_string());

        let trophy1 = Trophy::new(
            1,
            "Bronze Trophy".to_string(),
            "Description".to_string(),
            TrophyGrade::Bronze,
        );
        let trophy2 = Trophy::new(
            2,
            "Silver Trophy".to_string(),
            "Description".to_string(),
            TrophyGrade::Silver,
        );

        set.add_trophy(trophy1);
        set.add_trophy(trophy2);

        assert_eq!(set.total_trophies(), 2);
        assert_eq!(set.unlocked_count(), 0);

        set.unlock_trophy(1);
        assert_eq!(set.unlocked_count(), 1);
        assert_eq!(set.progress_percentage(), 50.0);
    }

    #[test]
    fn test_trophy_manager() {
        let temp_dir = std::env::temp_dir().join("test_trophies");
        let manager = TrophyManager::new(temp_dir.clone());

        // Clean up before test
        let _ = std::fs::remove_dir_all(&temp_dir);

        let mut set = TrophySet::new("BLES00000".to_string(), "Test Game".to_string());
        set.add_trophy(Trophy::new(
            1,
            "Test Trophy".to_string(),
            "Description".to_string(),
            TrophyGrade::Bronze,
        ));

        manager.register_set(set);
        manager.unlock_trophy("BLES00000", 1).unwrap();

        let loaded_set = manager.get_set("BLES00000").unwrap();
        assert!(loaded_set.is_unlocked(1));

        // Clean up after test
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
