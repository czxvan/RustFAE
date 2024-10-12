use std::process::{Command, exit};
pub fn extract_firmware(firmware: &str, directory: &str) {
    // Check if binwalk is installed
    if Command::new("binwalk").arg("-h").output().is_err() {
        eprintln!("binwalk is not installed. Please install it and try again.");
        exit(1);
    }

    // Run binwalk to extract the firmware
    let output = Command::new("binwalk")
        .args(&["--extract", "--directory", directory, firmware])
        .output()
        .expect("Failed to execute binwalk");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}