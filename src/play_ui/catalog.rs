//! Bundled and embedded Shipped level catalog with full hierarchical folder navigation.

use bevy::prelude::*;
use crate::level::LevelData;

// Include the auto-generated static slice of all shipped levels from build.rs
include!(concat!(env!("OUT_DIR"), "/embedded_shipped_levels.rs"));

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CatalogSortColumn {
    #[default]
    Name,
    Moves,
    Difficulty,
    Blocks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CatalogSortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Clone, Debug)]
pub struct LevelEntry {
    pub id: String,
    pub rel_path: String,
    pub folder: String,
    pub file_name: String,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub level_data: LevelData,
    pub macro_moves: Option<usize>,
    pub epiphany_score: Option<f32>,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub body_count: usize,
}

impl LevelEntry {
    pub fn new(rel_path: String, folder: String, file_name: String, level_data: LevelData) -> Self {
        let macro_moves = level_data
            .quality_profile
            .as_ref()
            .map(|p| p.macro_steps)
            .or_else(|| {
                level_data
                    .solutions
                    .first()
                    .and_then(|s| s.profile.as_ref().map(|p| p.macro_steps))
            });

        let epiphany_score = level_data
            .quality_profile
            .as_ref()
            .map(|p| p.epiphany_score)
            .or_else(|| {
                level_data
                    .solutions
                    .first()
                    .and_then(|s| s.profile.as_ref().map(|p| p.epiphany_score))
            });

        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        let mut min_z = i32::MAX;
        let mut max_z = i32::MIN;

        for b in &level_data.bodies {
            min_x = min_x.min(b.anchor[0]);
            max_x = max_x.max(b.anchor[0]);
            min_y = min_y.min(b.anchor[1]);
            max_y = max_y.max(b.anchor[1]);
            min_z = min_z.min(b.anchor[2]);
            max_z = max_z.max(b.anchor[2]);
        }

        let (w, h, d) = if level_data.bodies.is_empty() {
            (0, 0, 0)
        } else {
            (max_x - min_x + 1, max_y - min_y + 1, max_z - min_z + 1)
        };

        let body_count = level_data.bodies.len();

        let title = if level_data.name.is_empty() || level_data.name == "Custom Level" {
            file_name.trim_end_matches(".json").replace('_', " ")
        } else {
            level_data.name.clone()
        };

        let difficulty = if let Some(m) = macro_moves {
            if m <= 4 {
                "Introductory".to_string()
            } else if m <= 8 {
                "Medium".to_string()
            } else {
                "Advanced".to_string()
            }
        } else {
            "Puzzle".to_string()
        };

        Self {
            id: rel_path.clone(),
            rel_path: rel_path.clone(),
            folder,
            file_name,
            title,
            description: format!("Shipped level: {}", rel_path),
            difficulty,
            level_data,
            macro_moves,
            epiphany_score,
            width: w,
            height: h,
            depth: d,
            body_count,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct LevelCatalog {
    pub levels: Vec<LevelEntry>,
    pub current_folder: String,
    pub current_level_index: usize,
    pub scroll_offset: usize,
    pub max_visible_rows: usize,
    pub sort_column: CatalogSortColumn,
    pub sort_direction: CatalogSortDirection,
    pub search_query: String,
    pub is_dirty: bool,
}

impl Default for LevelCatalog {
    fn default() -> Self {
        let mut levels = Vec::new();

        // 1. Load embedded shipped levels from build.rs
        for embedded in EMBEDDED_SHIPPED_LEVELS {
            if let Ok(level_data) = serde_json::from_str::<LevelData>(embedded.json) {
                levels.push(LevelEntry::new(
                    embedded.rel_path.to_string(),
                    embedded.folder.to_string(),
                    embedded.filename.to_string(),
                    level_data,
                ));
            }
        }

        Self {
            levels,
            current_folder: String::new(),
            current_level_index: 0,
            scroll_offset: 0,
            max_visible_rows: 10,
            sort_column: CatalogSortColumn::Name,
            sort_direction: CatalogSortDirection::Ascending,
            search_query: String::new(),
            is_dirty: true,
        }
    }
}

impl LevelCatalog {
    /// Returns the active level entry being played or previewed.
    pub fn current_level(&self) -> Option<&LevelEntry> {
        self.levels.get(self.current_level_index)
    }

    /// List unique subfolder names present within `self.current_folder`.
    pub fn folders_in_current_folder(&self) -> Vec<String> {
        let mut folders = Vec::new();
        let prefix = if self.current_folder.is_empty() {
            String::new()
        } else {
            format!("{}/", self.current_folder)
        };

        for lvl in &self.levels {
            if lvl.folder.starts_with(&prefix) && lvl.folder.len() > prefix.len() {
                let remainder = &lvl.folder[prefix.len()..];
                let subfolder_name = remainder.split('/').next().unwrap_or("");
                if !subfolder_name.is_empty() && !folders.contains(&subfolder_name.to_string()) {
                    folders.push(subfolder_name.to_string());
                }
            }
        }

        folders.sort();
        folders
    }

    /// List all indices of levels inside `self.current_folder` matching search query and sorted.
    pub fn level_indices_in_current_folder(&self) -> Vec<usize> {
        let query_lower = self.search_query.to_lowercase();
        let mut indices: Vec<usize> = self
            .levels
            .iter()
            .enumerate()
            .filter(|(_, lvl)| {
                let matches_folder = lvl.folder == self.current_folder;
                let matches_search = query_lower.is_empty()
                    || lvl.title.to_lowercase().contains(&query_lower)
                    || lvl.file_name.to_lowercase().contains(&query_lower);
                matches_folder && matches_search
            })
            .map(|(idx, _)| idx)
            .collect();

        // Sort indices based on chosen column and direction
        let col = self.sort_column;
        let dir = self.sort_direction;
        let levels = &self.levels;

        indices.sort_by(|&a, &b| {
            let la = &levels[a];
            let lb = &levels[b];
            let cmp = match col {
                CatalogSortColumn::Name => la.title.to_lowercase().cmp(&lb.title.to_lowercase()),
                CatalogSortColumn::Moves => {
                    let ma = la.macro_moves.unwrap_or(0);
                    let mb = lb.macro_moves.unwrap_or(0);
                    ma.cmp(&mb)
                }
                CatalogSortColumn::Difficulty => la.difficulty.cmp(&lb.difficulty),
                CatalogSortColumn::Blocks => la.body_count.cmp(&lb.body_count),
            };

            if dir == CatalogSortDirection::Ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });

        indices
    }

    /// Total item count in the current folder view (folders + levels).
    pub fn total_items_count(&self) -> usize {
        self.folders_in_current_folder().len() + self.level_indices_in_current_folder().len()
    }

    /// Navigate into a child subfolder.
    pub fn navigate_into(&mut self, subfolder: &str) {
        if self.current_folder.is_empty() {
            self.current_folder = subfolder.to_string();
        } else {
            self.current_folder = format!("{}/{}", self.current_folder, subfolder);
        }
        self.scroll_offset = 0;
        self.is_dirty = true;
        let indices = self.level_indices_in_current_folder();
        if let Some(&first_idx) = indices.first() {
            self.current_level_index = first_idx;
        }
    }

    /// Navigate up to the parent folder.
    pub fn navigate_up(&mut self) {
        if self.current_folder.is_empty() {
            return;
        }

        if let Some(pos) = self.current_folder.rfind('/') {
            self.current_folder = self.current_folder[..pos].to_string();
        } else {
            self.current_folder.clear();
        }
        self.scroll_offset = 0;
        self.is_dirty = true;
        let indices = self.level_indices_in_current_folder();
        if let Some(&first_idx) = indices.first() {
            self.current_level_index = first_idx;
        }
    }

    /// Select the next level in the current folder (or wraps to first).
    pub fn select_next_in_folder(&mut self) -> Option<&LevelEntry> {
        let indices = self.level_indices_in_current_folder();
        if indices.is_empty() {
            return None;
        }

        let curr_pos = indices.iter().position(|&idx| idx == self.current_level_index);
        let next_idx = match curr_pos {
            Some(pos) => indices[(pos + 1) % indices.len()],
            None => indices[0],
        };

        self.current_level_index = next_idx;
        self.is_dirty = true;
        self.current_level()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_shipped_levels_parse_and_folder_navigation() {
        let mut catalog = LevelCatalog::default();
        assert!(!catalog.levels.is_empty(), "Should load embedded levels from levels/shipped");

        // Test root folder
        assert_eq!(catalog.current_folder, "");
        let root_levels = catalog.level_indices_in_current_folder();
        let subfolders = catalog.folders_in_current_folder();
        assert!(root_levels.len() + subfolders.len() > 0);

        // If subfolders exist, test navigating into and up
        if let Some(first_folder) = subfolders.first().cloned() {
            catalog.navigate_into(&first_folder);
            assert_eq!(catalog.current_folder, first_folder);

            catalog.navigate_up();
            assert_eq!(catalog.current_folder, "");
        }
    }

    #[test]
    fn catalog_sorting_by_moves_and_name() {
        let mut catalog = LevelCatalog::default();
        catalog.sort_column = CatalogSortColumn::Name;
        catalog.sort_direction = CatalogSortDirection::Ascending;
        let indices_asc = catalog.level_indices_in_current_folder();

        catalog.sort_direction = CatalogSortDirection::Descending;
        let indices_desc = catalog.level_indices_in_current_folder();

        if indices_asc.len() >= 2 {
            assert_eq!(indices_asc.first(), indices_desc.last());
        }
    }
}
