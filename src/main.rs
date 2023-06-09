use clap::Parser;
use std::fs::File;
use std::process;
use std::io::{Read, Write};
use hex::decode;
use std::path::Path;

#[derive(Debug, Parser)]
#[clap(name = "image analyzer")]
pub struct ImageAnalyzer {
    /// The input image path
    image: String,
    #[clap(short = 't', long, default_value="jpg")]
    /// The image type only support png and jpg
    image_type: String,

    #[clap(short, long)]
    /// The output image image path
    output: Option<String>,

    #[clap(short, long)]
    height: Option<u16>,
    #[clap(short, long)]
    width: Option<u16>,
}

fn main() {
    let args: ImageAnalyzer = ImageAnalyzer::parse();    

    match args.image_type.as_str() {
        "png" => println!("PNG image"),
        "jpg" => modify_jpg(&args),
        _ => println!("Unknown image type: {}", args.image_type),
    }
}

fn modify_jpg(args: &ImageAnalyzer) {
    let image_path = &args.image;
    let mut file = match File::open(image_path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error while opening the file: {}", error);
            process::exit(1);
        }
    };

    let mut buffer = Vec::new();

    if let Err(error) = file.read_to_end(&mut buffer) {
        eprintln!("Error while reading the file: {}", error);
        process::exit(1);
    }

    let marker = match decode("FFC0") {
        Ok(marker) => marker,
        Err(error) => {
            eprintln!("Error while decoding the marker: {}", error);
            process::exit(1);
        }
    };


    let position = match find_sequence(&buffer, &marker) {
        Some(position) => position,
        None => {
            eprintln!("Error while finding the marker");
            process::exit(1);
        }
    };

    println!("Found the marker at position {}", position);

    let height_index = position + 5; // marker + length + precision
    let width_index = height_index + 2;

    let modified_height = match args.height {
        Some(height) => height,
        None => u16::from_be_bytes([buffer[height_index], buffer[height_index+1]]),
    };

    let modified_width = match args.width {
        Some(width) => width,
        None => u16::from_be_bytes([buffer[width_index], buffer[width_index+1]]),
    };

    buffer[height_index..height_index+2].copy_from_slice(&modified_height.to_be_bytes());
    buffer[width_index..width_index+2].copy_from_slice(&modified_width.to_be_bytes());

    let original_path = Path::new(image_path);
    let new_path = original_path.with_file_name(format!(
        "{}_modified{}",
        original_path.file_stem().unwrap().to_string_lossy(),
        original_path.extension().map(|os_str| format!(".{}", os_str.to_string_lossy())).unwrap_or_default()
    ));
    let binding = new_path.to_str().unwrap();

    let output = match &args.output {
        Some(output) => output,
        // image path with extension replaced by _modified.extension
        None => binding, 
    };

    let mut output_file = match File::create(output) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error while creating the output file: {}", error);
            process::exit(1);
        }
    };

    if let Err(error) = output_file.write_all(&buffer) {
        eprintln!("Error while writing the output file: {}", error);
        process::exit(1);
    }
}
fn find_sequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
