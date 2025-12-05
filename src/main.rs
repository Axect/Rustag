use core::ops::Deref;
use std::{
    env::current_dir,
    result::Result,
    collections::HashMap,
    error::Error,
    path::Path,
};
use shellexpand::env;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use postcard::{from_bytes, to_allocvec};
use dialoguer::{theme::ColorfulTheme, Input, FuzzySelect};

const ROOT: &str = "$HOME/.rustag/";
const BOOKMARKFILE: &str = "bookmarks";

fn main() -> Result<(), Box<dyn Error>> {
    // Check if root directory exists
    let root = env(ROOT).unwrap();
    let root_dir = Path::new(root.as_ref());
    let bookmarkfile_path = root_dir.join(BOOKMARKFILE);

    // Create directory if it doesn't exist
    if !root_dir.exists() {
        std::fs::create_dir_all(root_dir)?;
    }

    // Create default bookmarks file if it doesn't exist
    if !bookmarkfile_path.exists() {
        let bookmark_list = BookmarkList::default();
        let write_buffer = to_allocvec(&bookmark_list)?;
        std::fs::write(bookmarkfile_path.as_path(), write_buffer)?;
    }

    // Read bookmark list
    let read_buffer = std::fs::read(bookmarkfile_path.as_path())?;
    let mut bookmark_list: BookmarkList = from_bytes(read_buffer.deref())?;

    // Main menu
    let menu_items = vec!["Add bookmark", "View bookmarks"];
    let menu_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select menu")
        .items(&menu_items)
        .default(0)
        .interact()?;

    match menu_selection {
        0 => {
            // Add bookmark
            add_bookmark(&mut bookmark_list, &bookmarkfile_path)?;
        }
        1 => {
            // View bookmarks
            view_bookmarks(&mut bookmark_list, &bookmarkfile_path)?;
        }
        _ => {
            println!("{}", env("$PWD").unwrap());
        }
    }

    Ok(())
}

fn add_bookmark(
    bookmark_list: &mut BookmarkList,
    bookmarkfile_path: &Path,
) -> Result<(), Box<dyn Error>> {
    // Get current directory
    let current_dir = current_dir()?;

    // Validate that current directory exists and is a directory
    if !current_dir.exists() {
        return Err("Current directory does not exist".into());
    }
    if !current_dir.is_dir() {
        return Err("Current path is not a directory".into());
    }

    // Canonicalize the path
    let canonical_path = std::fs::canonicalize(&current_dir)?;
    let folder_path = canonical_path
        .to_str()
        .ok_or("Path contains invalid UTF-8 characters")?
        .to_string();

    // Input alias with validation
    let alias: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter alias")
        .validate_with(|input: &String| -> Result<(), &str> {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                Err("Alias cannot be empty")
            } else if trimmed.contains('/') || trimmed.contains('\\') {
                Err("Alias cannot contain path separators")
            } else if bookmark_list.exists(trimmed) {
                Err("Alias already exists")
            } else {
                Ok(())
            }
        })
        .interact()?;

    // Create bookmark
    let bookmark = Bookmark {
        alias: alias.trim().to_string(),
        folder_path: folder_path.clone(),
        created_at: Utc::now(),
        last_accessed: None,
    };

    // Insert bookmark
    bookmark_list.insert_bookmark(bookmark)?;

    // Save bookmarks
    save_bookmarks(bookmark_list, bookmarkfile_path)?;

    println!("Bookmark '{}' added: {}", alias.trim(), folder_path);
    println!("{}", env("$PWD").unwrap());

    Ok(())
}

fn view_bookmarks(
    bookmark_list: &mut BookmarkList,
    bookmarkfile_path: &Path,
) -> Result<(), Box<dyn Error>> {
    // Check if there are any bookmarks
    if bookmark_list.is_empty() {
        println!("No bookmarks found. Please add a bookmark first.");
        println!("{}", env("$PWD").unwrap());
        return Ok(());
    }

    // Create formatted menu items with alias and path
    let aliases = bookmark_list.get_aliases();
    let menu_items: Vec<String> = aliases
        .iter()
        .filter_map(|alias| {
            bookmark_list.get_bookmark(alias).map(|bookmark| {
                format!("{} -> {}", alias, bookmark.folder_path)
            })
        })
        .collect();

    // Select bookmark
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select bookmark")
        .items(&menu_items)
        .default(0)
        .interact()?;

    let selected_alias = &aliases[selection].clone();
    let bookmark = bookmark_list
        .get_bookmark(selected_alias)
        .ok_or("Bookmark not found")?
        .clone();

    // Check if bookmark path still exists
    let bookmark_path = Path::new(&bookmark.folder_path);
    if !bookmark_path.exists() {
        println!("Warning: Path no longer exists: {}", bookmark.folder_path);
        println!("Do you want to remove this bookmark?");

        let confirm = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Confirm deletion")
            .items(&["Yes", "No"])
            .default(1)
            .interact()?;

        if confirm == 0 {
            bookmark_list.remove_bookmark(selected_alias);
            save_bookmarks(bookmark_list, bookmarkfile_path)?;
            println!("Bookmark '{}' removed", selected_alias);
        }
        println!("{}", env("$PWD").unwrap());
        return Ok(());
    }

    // Action menu
    let actions = vec![
        "Open in Terminal",
        "Open in File Manager",
        "Edit alias",
        "Remove bookmark",
    ];

    let action_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select action")
        .items(&actions)
        .default(0)
        .interact()?;

    match action_selection {
        0 => {
            // Open in Terminal (print path for rtg wrapper to cd)
            bookmark_list.update_last_accessed(selected_alias);
            save_bookmarks(bookmark_list, bookmarkfile_path)?;
            println!("{}", bookmark.folder_path);
        }
        1 => {
            // Open in File Manager
            std::process::Command::new("xdg-open")
                .arg(&bookmark.folder_path)
                .spawn()?;
            println!("{}", env("$PWD").unwrap());
        }
        2 => {
            // Edit alias
            let new_alias: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter new alias")
                .validate_with(|input: &String| -> Result<(), &str> {
                    let trimmed = input.trim();
                    if trimmed.is_empty() {
                        Err("Alias cannot be empty")
                    } else if trimmed.contains('/') || trimmed.contains('\\') {
                        Err("Alias cannot contain path separators")
                    } else if bookmark_list.exists(trimmed) && trimmed != selected_alias {
                        Err("Alias already exists")
                    } else {
                        Ok(())
                    }
                })
                .interact()?;

            bookmark_list.update_alias(selected_alias, new_alias.trim())?;
            save_bookmarks(bookmark_list, bookmarkfile_path)?;
            println!("Alias changed from '{}' to '{}'", selected_alias, new_alias.trim());
            println!("{}", env("$PWD").unwrap());
        }
        3 => {
            // Remove bookmark
            bookmark_list.remove_bookmark(selected_alias);
            save_bookmarks(bookmark_list, bookmarkfile_path)?;
            println!("Bookmark '{}' removed", selected_alias);
            println!("{}", env("$PWD").unwrap());
        }
        _ => {
            println!("{}", env("$PWD").unwrap());
        }
    }

    Ok(())
}

fn save_bookmarks(
    bookmark_list: &BookmarkList,
    bookmarkfile_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let write_buffer = to_allocvec(bookmark_list)?;
    std::fs::write(bookmarkfile_path, write_buffer)?;
    Ok(())
}

// Data Structures

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Bookmark {
    alias: String,
    folder_path: String,
    created_at: DateTime<Utc>,
    last_accessed: Option<DateTime<Utc>>,
}

impl Bookmark {
    pub fn get_alias(&self) -> &str {
        &self.alias
    }

    pub fn get_folder_path(&self) -> &str {
        &self.folder_path
    }

    pub fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn get_last_accessed(&self) -> &Option<DateTime<Utc>> {
        &self.last_accessed
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BookmarkList {
    bookmarks: HashMap<String, Bookmark>,
    aliases: Vec<String>,
}

impl BookmarkList {
    pub fn get_aliases(&self) -> &Vec<String> {
        &self.aliases
    }

    pub fn get_bookmark(&self, alias: &str) -> Option<&Bookmark> {
        self.bookmarks.get(alias)
    }

    pub fn get_bookmark_mut(&mut self, alias: &str) -> Option<&mut Bookmark> {
        self.bookmarks.get_mut(alias)
    }

    pub fn insert_bookmark(&mut self, bookmark: Bookmark) -> Result<(), String> {
        if self.bookmarks.contains_key(&bookmark.alias) {
            return Err(format!("Alias '{}' already exists", bookmark.alias));
        }

        self.aliases.push(bookmark.alias.clone());
        self.bookmarks.insert(bookmark.alias.clone(), bookmark);
        self.sort_aliases();
        Ok(())
    }

    pub fn remove_bookmark(&mut self, alias: &str) {
        self.bookmarks.remove(alias);
        self.aliases.retain(|a| a != alias);
    }

    pub fn update_alias(&mut self, old_alias: &str, new_alias: &str) -> Result<(), String> {
        if old_alias == new_alias {
            return Ok(());
        }

        if self.bookmarks.contains_key(new_alias) {
            return Err(format!("Alias '{}' already exists", new_alias));
        }

        if let Some(mut bookmark) = self.bookmarks.remove(old_alias) {
            bookmark.alias = new_alias.to_string();
            self.bookmarks.insert(new_alias.to_string(), bookmark);

            // Update aliases vector
            if let Some(pos) = self.aliases.iter().position(|a| a == old_alias) {
                self.aliases[pos] = new_alias.to_string();
            }
            self.sort_aliases();
            Ok(())
        } else {
            Err(format!("Alias '{}' not found", old_alias))
        }
    }

    pub fn update_last_accessed(&mut self, alias: &str) {
        if let Some(bookmark) = self.bookmarks.get_mut(alias) {
            bookmark.last_accessed = Some(Utc::now());
        }
    }

    pub fn exists(&self, alias: &str) -> bool {
        self.bookmarks.contains_key(alias)
    }

    pub fn is_empty(&self) -> bool {
        self.bookmarks.is_empty()
    }

    fn sort_aliases(&mut self) {
        self.aliases.sort();
    }
}
