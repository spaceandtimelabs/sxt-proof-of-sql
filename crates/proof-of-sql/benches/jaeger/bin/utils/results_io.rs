use csv::{Writer, WriterBuilder};
use std::{fs::OpenOptions, io::BufWriter, path::Path};

/// Writes the header to the CSV file.
///
/// # Arguments
/// * `writer` - A mutable reference to the CSV writer.
///
/// # Panics
/// * If the header cannot be written to the CSV file.
fn write_csv_header(writer: &mut Writer<BufWriter<std::fs::File>>) {
    writer
        .write_record([
            "commitment_scheme",
            "query",
            "table_size",
            "generate_proof (ms)",
            "verify_proof (ms)",
            "iteration",
        ])
        .expect("Failed to write headers to CSV file.");
}

/// Appends values to an existing CSV file or creates a new one if it doesn't exist.
///
/// # Arguments
/// * `file_path` - The path to the CSV file.
/// * `new_row` - A vector of strings to append to the file.
///
/// # Panics
/// * If the file cannot be opened, read, or appended.
pub fn append_to_csv(file_path: &Path, new_row: &[String]) {
    // Open the file in append mode or create it if it doesn't exist
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(file_path)
        .expect("Failed to open or create the CSV file.");

    // Check if the file is empty to determine if we need to write headers
    let is_empty = file.metadata().map(|m| m.len() == 0).unwrap_or(true);

    // Create a CSV writer
    let mut writer = WriterBuilder::new().from_writer(BufWriter::new(file));

    // Write headers if the file is empty
    if is_empty {
        write_csv_header(&mut writer);
    }

    // Write new row to the CSV file
    writer
        .write_record(new_row)
        .expect("Failed to write row to CSV file.");

    writer.flush().expect("Failed to flush CSV writer.");
}
