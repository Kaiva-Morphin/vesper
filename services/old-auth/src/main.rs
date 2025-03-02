use std::path::Path;

fn main() {
    stringify!("");
    // Get the path to the directory where Cargo.toml is located
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // Combine it with the relative path to the file you want to include
    let file_path = Path::new(&manifest_dir).join("path/to/your/file.txt");

    // Now you can use the file_path, e.g., opening the file or using include_bytes
    use std::fs;

    let file_contents = fs::read(file_path).expect("Unable to read file");
    println!("File contents: {:?}", file_contents);
}