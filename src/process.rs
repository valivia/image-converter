use std::{
    error::Error,
    fs::{self},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use image::{
    codecs::{avif::AvifEncoder, jpeg::JpegEncoder},
    imageops::FilterType,
    GenericImageView,
};

use crate::{
    structs::{
        file_type::EncodingOptions,
        settings::{ResizeOptions, Settings},
    },
    types::{Message, Progress},
};

static INPUT_FOLDER: &str = "input";
static OUTPUT_FOLDER: &str = "output";

pub fn convert_images(
    sender: std::sync::mpsc::Sender<Message>,
    stop_flag: Arc<AtomicBool>,
    settings: Settings,
) {
    let files = match get_files() {
        Ok(files) => files,
        Err(e) => {
            println!("Failed to get files: {}", e);
            sender
                .send(Message::Failed("Failed to get files".to_string()))
                .unwrap();
            return;
        }
    };

    sender
        .send(Message::Message(format!(
            "Processing {} files...",
            files.len()
        )))
        .unwrap();

    let mut progress = Progress::new(files.len() as u32);
    sender.send(Message::Progress(progress.clone())).unwrap();

    for file in files {
        let start_time = std::time::Instant::now();
        if stop_flag.load(Ordering::Relaxed) {
            sender.send(Message::Completed).unwrap();
            return;
        }

        let file_name = file.file_name().unwrap().to_str().unwrap();

        sender
            .send(Message::Message(format!("Processing '{}'...", &file_name,)))
            .unwrap();

        match convert_image(&file, &settings) {
            Ok(_) => {
                let elapsed = start_time.elapsed().as_secs_f32();
                progress.increment_success();
                sender
                    .send(Message::Message(format!(
                        "Processed '{}' in {:.2} seconds",
                        &file_name, elapsed
                    )))
                    .unwrap();
            }
            Err(e) => {
                eprintln!("Failed to process '{}': {}", file_name, e);
                progress.increment_failed();
                sender
                    .send(Message::Warning(format!(
                        "Failed to process '{}'",
                        &file_name,
                    )))
                    .unwrap();
            }
        }
        sender.send(Message::Progress(progress.clone())).unwrap();
    }

    sender.send(Message::Completed).unwrap();
}

fn get_files() -> Result<Vec<PathBuf>, Box<dyn Error>> {
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

fn convert_image(path: &Path, settings: &Settings) -> Result<(), Box<dyn Error>> {
    let img = get_image(path)?;
    let img = resize_image(img, settings);
    let data = encode_image(img, settings)?;
    save_image(&data, path, settings)?;
    Ok(())
}

fn get_image(image_path: &Path) -> Result<image::DynamicImage, Box<dyn Error>> {
    image::open(image_path).map_err(|e| {
        eprintln!("Failed to open image '{}': {}", image_path.display(), e);
        e.into()
    })
}

fn resize_image(img: image::DynamicImage, settings: &Settings) -> image::DynamicImage {
    let (width, height) = img.dimensions();

    match settings.resize_options {
        ResizeOptions::Smallest(size) => {
            let new_width = if width < height {
                size
            } else {
                size * width / height
            };
            let new_height = if height < width {
                size
            } else {
                size * height / width
            };
            img.resize(new_width, new_height, FilterType::Lanczos3)
        }

        ResizeOptions::Exact(new_width, new_height) => {
            img.resize_to_fill(new_width, new_height, FilterType::Lanczos3)
        }

        ResizeOptions::Largest(size) => {
            let new_width = if width > height {
                size
            } else {
                size * width / height
            };
            let new_height = if height > width {
                size
            } else {
                size * height / width
            };

            img.resize(new_width, new_height, FilterType::Lanczos3)
        }

        // No resize
        ResizeOptions::None => img,
    }
}

fn encode_image(img: image::DynamicImage, settings: &Settings) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = Vec::new();

    let data = match &settings.encoding_options {
        // Webp
        EncodingOptions::WebP(options) => {
            let encoder = webp::Encoder::from_image(&img).inspect_err(|&e| {
                eprintln!("Failed to encode image: {}", e);
            })?;

            let buffer = match options.lossless {
                true => encoder.encode_lossless().to_vec(),
                false => encoder.encode(options.quality as f32).to_vec(),
            };

            buffer.to_vec()
        }

        // Avif
        EncodingOptions::Avif(options) => {
            img.write_with_encoder(AvifEncoder::new_with_speed_quality(
                &mut buf,
                options.speed,
                options.quality,
            ))
            .map_err(|e| format!("Failed to encode AVIF: {}", e))?;
            buf
        }

        // Jpeg
        EncodingOptions::Jpeg(options) => {
            img.write_with_encoder(JpegEncoder::new_with_quality(&mut buf, options.quality))
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
            buf
        }
    };

    Ok(data)
}

fn save_image(data: &[u8], image_path: &Path, settings: &Settings) -> Result<(), Box<dyn Error>> {
    let mut output_file_name = image_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| {
            eprintln!(
                "Failed to extract file stem from '{}'",
                image_path.display()
            );
            "Invalid file stem"
        })?
        .to_owned();

    if let Some(name_extension) = &settings.name_extension {
        output_file_name.push_str(name_extension);
    }

    let extension = match settings.encoding_options {
        EncodingOptions::WebP(_) => ".webp",
        EncodingOptions::Avif(_) => ".avif",
        EncodingOptions::Jpeg(_) => ".jpg",
    };

    output_file_name.push_str(extension);

    let output_file_path = Path::new(OUTPUT_FOLDER).join(output_file_name);

    // Attempt to write the file
    fs::write(&output_file_path, data).map_err(|e| {
        eprintln!(
            "Failed to write output file '{}': {}",
            output_file_path.display(),
            e
        );
        e
    })?;
    Ok(())
}
