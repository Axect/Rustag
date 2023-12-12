use core::ops::Deref;
use std::{
    env::args,
    result::Result,
    collections::HashMap,
    error::Error,
};
use shellexpand::env;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use postcard::{from_bytes, to_allocvec};
use dialoguer::{theme::ColorfulTheme, Input, FuzzySelect};

const ROOT: &str = "$HOME/.rustag/";
const TAGFILE: &str = "tagfile";

fn main() -> Result<(), Box<dyn Error>> {
    // Check if root directory exists
    let root = env(ROOT).unwrap();
    let root_dir = std::path::Path::new(root.as_ref());
    let tagfile_dir = root_dir.join(TAGFILE);

    if !root_dir.exists() {
        std::fs::create_dir_all(root_dir)?;

        // Create default AGTagList
        let tag_list = AGTagList::default();

        // Write default AGTagList
        let write_buffer = to_allocvec(&tag_list)?;
        std::fs::write(tagfile_dir.as_path(), write_buffer)?;
    }

    // Read taglist
    let read_buffer = std::fs::read(tagfile_dir.as_path())?;
    let mut tag_list: AGTagList = from_bytes(read_buffer.deref())?;

    // Receive file name
    let file_name = args().nth(1);

    // Check if file name is provided
    match file_name {
        None => {
            // No file name -> Choose tag -> Choose file to open
            // 0. Choose tag or remove tag
            let action = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose tag or remove tag")
                .items(&["Choose tag", "Remove tag"])
                .default(0)
                .interact()?;

            // 1. Choose tag
            let tags = tag_list.get_tags();
            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose tag")
                .items(tags)
                .default(0)
                .interact()?;
            let tag = &tags[selection];

            // 1.1. Remove tag
            if action == 1 {
                let mut tag_list = tag_list.clone();
                tag_list.remove_tag(tag);

                // Dump
                let write_buffer = to_allocvec(&tag_list)?;
                std::fs::write(tagfile_dir.as_path(), write_buffer)?;

                println!("{}", env("$PWD").unwrap());
                return Ok(());
            }

            // 2. Choose file
            let file = if let Some(files) = tag_list.get_files(tag) {
                let file_names = files.iter().map(|file| file.get_file_name()).collect::<Vec<&str>>();
                let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("Choose file")
                    .items(&file_names)
                    .default(0)
                    .interact()?;
                &files[selection]
            } else {
                panic!("Error: No files found in tag {}", tag);
            };

            // 3. Open file or open Path or remove file in tag
            let file_path = std::path::Path::new(file.get_file_path());
            if file_path.exists() {
                let (file_parent_dir, options, is_file) = if file_path.is_file() {
                    (file_path.parent().unwrap(), vec!["Open path in Terminal", "Open path in File Manager", "Open file", "Remove file in tag"], true)
                } else {
                    (file_path, vec!["Open path in Terminal", "Open path in File Manager", "Remove file in tag"], false)  
                };
                let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("Choose action")
                    .items(&options)
                    .default(0)
                    .interact()?;

                match selection {
                    0 => {
                        // Print path
                        println!("{}", file_parent_dir.display());
                    }
                    1 => {
                        // Open path in File Manager
                        std::process::Command::new("xdg-open")
                            .arg(file_parent_dir)
                            .spawn()?;
                        println!("{}", env("$PWD").unwrap());
                    }
                    2 => {
                        if is_file {
                            std::process::Command::new("xdg-open").arg(file_path).spawn()?;
                        } else {
                            let mut tag_list = tag_list.clone();
                            tag_list.remove_file(tag, file.get_file_name());
                            let write_buffer = to_allocvec(&tag_list)?;
                            std::fs::write(tagfile_dir.as_path(), write_buffer)?;
                        }
                        println!("{}", env("$PWD").unwrap());
                    }
                    3 => {
                        let mut tag_list = tag_list.clone();
                        tag_list.remove_file(tag, file.get_file_name());
                        let write_buffer = to_allocvec(&tag_list)?;
                        std::fs::write(tagfile_dir.as_path(), write_buffer)?;
                    }
                    _ => {
                        panic!("Error: Invalid selection");
                    }
                }
            } else {
                panic!("Error: {} is not found", file.get_file_name());
            }
        }

        Some(name) => {
            // Check if file exists
            let file_path_str = format!("{}/{}", env("$PWD").unwrap(), name);
            let file_path = std::path::Path::new(&file_path_str);
            if !file_path.exists() {
                panic!("Error: {} is not found", name);
            }

            // Create AGFile
            let mut file = AGFile {
                file_name: name.to_string(),
                file_path: file_path_str.clone(),
                created_at: Utc::now(),
                tags: vec![],
            };

            // Choose tags
            let mut options = tag_list.get_tags().clone();
            options.push("Create new tag".to_string());

            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose tags")
                .items(&options)
                .default(0)
                .interact()?;

            if selection == options.len() - 1 {
                // Create new tag
                let tag_name: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter tag name")
                    .interact()?;

                file.tags.push(tag_name.clone());
            } else {
                file.tags.push(options[selection].clone());
            }

            // Insert file
            tag_list.insert_file(file);

            // Write taglist
            let write_buffer = to_allocvec(&tag_list)?;
            std::fs::write(tagfile_dir.as_path(), write_buffer)?;

            println!("{}", env("$PWD").unwrap());
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AGFile {
    file_name: String,
    file_path: String,
    created_at: DateTime<Utc>,
    tags: Vec<String>,
}

impl AGFile {
    pub fn get_file_name(&self) -> &str {
        &self.file_name
    }

    pub fn get_file_path(&self) -> &str {
        &self.file_path
    }

    pub fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn get_tags(&self) -> &Vec<String> {
        &self.tags
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AGTag {
    tag_name: String,
    files: Vec<AGFile>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AGTagList {
    tags: Vec<String>,
    tag_map: HashMap<String, AGTag>,
}

impl AGTagList {
    pub fn get_tags(&self) -> &Vec<String> {
        &self.tags
    }

    pub fn get_files(&self, tag: &str) -> Option<&Vec<AGFile>> {
        self.tag_map.get(tag).map(|tag| &tag.files)
    }

    pub fn get_mut_files(&mut self, tag: &str) -> Option<&mut Vec<AGFile>> {
        self.tag_map.get_mut(tag).map(|tag| &mut tag.files)
    }

    pub fn insert_file(&mut self, file: AGFile) {
        for tag in file.tags.iter() {
            let is_new_tag = self.tag_map.get(tag).is_none();
            let files = &mut self.tag_map
                .entry(tag.to_string())
                .or_insert(AGTag {
                    tag_name: tag.to_string(),
                    files: vec![],
                })
                .files;
            if files.iter().any(|f| f.file_name == file.file_name) {
                println!("{} already exists in {}", file.file_name, tag);
                continue
            } else {
                files.push(file.clone());
                if is_new_tag {
                    self.tags.push(tag.to_string());
                }
            }
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.tag_map.remove(tag);
    }

    pub fn remove_file(&mut self, tag: &str, file_name: &str) {
        if let Some(files) = self.get_mut_files(tag) {
            files.retain(|f| f.file_name != file_name);
            if files.is_empty() {
                self.remove_tag(tag);
            }
        }
    }
}
