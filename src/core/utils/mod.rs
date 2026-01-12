// Utility functions for printing and formatting

pub fn print_header(title: &str) {
    println!("\n═══════════════════════════════════════");
    println!("{title}");
    println!("═══════════════════════════════════════\n");
}

pub fn print_step(msg: &str) {
    println!("➜ {msg}");
}

pub fn print_success(msg: &str) {
    println!("✓ {msg}");
}

pub fn print_info(msg: &str) {
    println!("ℹ {msg}");
}

pub fn print_warning(msg: &str) {
    println!("⚠ {msg}");
}
