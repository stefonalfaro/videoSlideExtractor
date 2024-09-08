use std::env;
use std::io::Error;
use std::io::ErrorKind;
use image::{DynamicImage, GenericImageView};
use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};

/// Extract frames from the video using ffmpeg
fn extract_frames(input_file: &str, output_dir: &str, fps: u32) -> Result<(), Error> {
    // Ensure output directory exists
    if !Path::new(output_dir).exists() {
        fs::create_dir(output_dir)?;
    }

    // Spawn ffmpeg process to extract frames
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_file)
        .arg("-vf")
        .arg(format!("fps={}", fps))  // Set the frame extraction rate
        .arg(format!("{}/frame_%04d.png", output_dir))  // Output pattern for frame files
        .status()?;

    if !status.success() {
        eprintln!("ffmpeg process failed");
    } else {
        println!("Frames extracted successfully.");
    }

    Ok(())
}

/// Compare two images and determine if they are visually similar
fn are_images_similar(img1: &DynamicImage, img2: &DynamicImage, threshold: f64) -> bool {
    if img1.dimensions() != img2.dimensions() {
        return false;
    }

    let (width, height) = img1.dimensions();
    let mut diff_count = 0;

    for x in 0..width {
        for y in 0..height {
            let p1 = img1.get_pixel(x, y);
            let p2 = img2.get_pixel(x, y);

            if p1 != p2 {
                diff_count += 1;
            }
        }
    }

    let total_pixels = width * height;
    let difference_ratio = (diff_count as f64) / (total_pixels as f64);
    
    difference_ratio <= threshold
}

/// Process extracted frames and filter out non-unique frames
fn process_frames(output_dir: &str, threshold: f64) -> Result<(), Error> {
    let mut frame_files: Vec<PathBuf> = fs::read_dir(output_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("png"))
        .map(|entry| entry.path())
        .collect();

    frame_files.sort(); // Ensure files are sorted in correct order

    let mut last_image: Option<DynamicImage> = None;

    for frame in frame_files {
        // Here, we map the image error to an io::Error
        let current_image = image::open(&frame).map_err(|e| {
            Error::new(ErrorKind::Other, format!("Error opening image: {}", e))
        })?;

        if let Some(ref last_image) = last_image {
            if are_images_similar(last_image, &current_image, threshold) {
                println!("Frame {:?} is similar to the previous one, deleting it.", frame);
                fs::remove_file(&frame)?; // Remove non-unique frame
            } else {
                println!("Frame {:?} is unique.", frame);
            }
        } else {
            println!("First frame {:?} is considered unique.", frame);
        }

        last_image = Some(current_image);
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    let file_path = Path::new(&args[1]);

    // Convert the file_path (Path) to a &str
    let input_file = match file_path.to_str() {
        Some(path_str) => path_str,  // Valid string path
        None => {
            eprintln!("Invalid file path.");
            std::process::exit(1);
        }
    };

    let output_dir = "frames";     // Directory to store extracted frames
    let fps = 1;                   // Set extraction to 1 frame per second (or as desired)
    let similarity_threshold = 0.01; // Threshold for image similarity (adjust as needed)

    // Step 1: Extract frames from the video
    extract_frames(input_file, output_dir, fps)?;

    // Step 2: Process the extracted frames and remove duplicates
    process_frames(output_dir, similarity_threshold)?;

    Ok(())
}
