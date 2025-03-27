use image::imageops::FilterType;
use image::GenericImageView;
use std::error::Error;
use std::fs::{self, create_dir_all};
use std::path::Path;
use std::{io, thread};
use webp::Encoder;

fn main() {
    let input_folder = "input"; // Change to your input folder path
    let output_folder = "output"; // Change to your output folder path

    // Quality
    println!("Enter quality value (0-100): ");
    let mut quality_input = String::new();
    io::stdin()
        .read_line(&mut quality_input)
        .expect("Failed to read line");
    let quality: u32 = quality_input
        .trim()
        .parse()
        .expect("Please enter a valid number between 0 and 100");

    // Check if the quality value is within range
    if quality > 100 {
        println!("Error: Quality value must be between 0 and 100.");
        return;
    }

    // Resolution
    println!("Enter resolution value (e.g. 1920, 1080, 720): ");
    let mut resolution_input = String::new();
    io::stdin()
        .read_line(&mut resolution_input)
        .expect("Failed to read line");
    let resolution: u32 = resolution_input
        .trim()
        .parse()
        .expect("Please enter a valid number");

    // Check if the resolution value is within range
    if resolution < 1 {
        println!("Error: Resolution value must be greater than 0.");
        return;
    }

    // Start time
    let start_time = std::time::Instant::now();

    println!(
        "Compressing images from '{}' to '{}'",
        input_folder, output_folder
    );

    if let Err(e) = compress_images_to_webp(input_folder, output_folder, resolution, quality) {
        eprintln!("Error: {}", e);
    }

    let elapsed = start_time.elapsed();
    println!(
        "Done in {}.{:03} seconds",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );

    println!("Press Enter to exit...");
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn compress_images_to_webp(
    input_folder: &str,
    output_folder: &str,
    size: u32,
    quality: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = Path::new(input_folder);
    let output_path = Path::new(output_folder);

    if !input_path.exists() || !input_path.is_dir() {
        return Err(format!(
            "Input folder '{}' does not exist or is not a directory.",
            input_folder
        )
        .into());
    }

    create_dir_all(output_path)?;

    let files = fs::read_dir(input_path)?;

    println!("Processing {} files...", files.size_hint().0);

    for entry in files {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "jpg" || extension == "jpeg" || extension == "png" {
                    if let Err(e) = process_image(&path, output_path, size, quality) {
                        eprintln!("Failed to process {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    Ok(())
}

fn process_image(
    input_path: &Path,
    output_path: &Path,
    size: u32,
    quality: u32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let input_path_clone = input_path.to_path_buf();
    let output_path_clone = output_path.to_path_buf();

    let handle = thread::spawn(move || {
        // Attempt to load the image
        let img = image::open(&input_path_clone).map_err(|e| {
            eprintln!(
                "Failed to open image '{}': {}",
                input_path_clone.display(),
                e
            );
            e
        })?;

        let (original_width, original_height) = img.dimensions();
        println!(
            "Processing '{}' ({}x{})",
            input_path_clone.display(),
            original_width,
            original_height
        );

        let (new_width, new_height) = if original_width <= size && original_height <= size {
            (original_width, original_height)
        } else if original_width > original_height {
            let new_width = size;
            let new_height =
                (new_width as f32 * original_height as f32 / original_width as f32) as u32;
            (new_width, new_height)
        } else {
            let new_height = size;
            let new_width =
                (new_height as f32 * original_width as f32 / original_height as f32) as u32;
            (new_width, new_height)
        };

        // Attempt to resize the image
        let resized_img = img.resize(new_width, new_height, FilterType::Lanczos3);

        // Attempt to create the output file name
        let output_file_name = input_path_clone
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| {
                eprintln!(
                    "Failed to extract file stem from '{}'",
                    input_path_clone.display()
                );
                "Invalid file stem"
            })?
            .to_owned()
            + ".webp";

        let output_file_path = output_path_clone.join(output_file_name);

        // Encode the image to WebP
        let encoder = Encoder::from_image(&resized_img).inspect_err(|&e| {
            eprintln!("Failed to encode image: {}", e);
        })?;
        let webp_data = encoder.encode(quality as f32);

        // Attempt to write the file
        fs::write(&output_file_path, &*webp_data).map_err(|e| {
            eprintln!(
                "Failed to write output file '{}': {}",
                output_file_path.display(),
                e
            );
            e
        })?;

        println!("Saved to '{}'", output_file_path.display());
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    });

    // Join the thread and propagate errors if any
    match handle.join() {
        Ok(result) => result,
        Err(_) => Err("Thread panicked while processing the image".into()),
    }
}
