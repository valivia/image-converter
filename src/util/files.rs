use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use crate::{INPUT_FOLDER, OUTPUT_FOLDER};

pub fn get_files() -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let input_path = Path::new(INPUT_FOLDER);
    let output_path = Path::new(OUTPUT_FOLDER);

    // Input folder
    if !input_path.exists() {
        println!("Creating input folder");
        fs::create_dir(input_path)?;
    } else if !input_path.is_dir() {
        return Err(format!("{} is not a directory", INPUT_FOLDER).into());
    }

    // Output folder
    if !output_path.exists() {
        println!("Creating output folder");
        fs::create_dir(output_path)?;
    } else if !output_path.is_dir() {
        return Err(format!("{} is not a directory", OUTPUT_FOLDER).into());
    }

    let allowed_extensions = ["jpg", "jpeg", "png", "avif"];

    // Get all image files
    let files: Vec<PathBuf> = fs::read_dir(input_path)?
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if let Some(ext) = extension.to_str().map(|ext| ext.to_ascii_lowercase()) {
                            if allowed_extensions.contains(&ext.as_str()) {
                                return Some(path);
                            }
                        }
                    }
                }
            }
            None
        })
        .collect();

    Ok(files)
}
