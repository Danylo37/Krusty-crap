use std::{thread, time, io::{stdout, Write}};

// ANSI Escape Codes for Colors
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m"; // Resets color

fn animate_message(msg: &str, frames: &[&str], delay: u64) {
    for frame in frames.iter() {
        print!("\r{}{} {}{}", CYAN, frame, msg, RESET);
        stdout().flush().unwrap(); // Forces output update
        thread::sleep(time::Duration::from_millis(delay));
    }
    println!();
}

fn fancy_box(title: &str, color: &str) {
    let width = title.len() + 6;
    let border = "━".repeat(width);

    println!(
        "\n{}┏{}┓\n┃  {}      ┃\n┗{}┛{}",
        color, border, title, border, RESET
    );
}

fn animated_progress_bar(task: &str, duration: u64) {
    let bar_frames = [
        "[■         ]", "[■■        ]", "[■■■       ]", "[■■■■      ]",
        "[■■■■■     ]", "[■■■■■■    ]", "[■■■■■■■   ]", "[■■■■■■■■  ]",
        "[■■■■■■■■■ ]", "[■■■■■■■■■■]"
    ];

    print!("{}{} ", CYAN, task);
    stdout().flush().unwrap(); // Ensure immediate output

    for frame in bar_frames.iter() {
        print!("\r{}{} {}", CYAN, task, frame);
        stdout().flush().unwrap();
        thread::sleep(time::Duration::from_millis(duration));
    }

    // Print the checkmark at the END of the line with no newline break
    print!("\r{}{} {} {}", CYAN, task, bar_frames.last().unwrap(), RESET);
    print!("✅ ");
    stdout().flush().unwrap(); // Ensure the checkmark is printed correctly
    println!(); // Move to the next line after completion
}

pub fn building_network(){
    let loading_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    // 🚀 Building the Network
    fancy_box("🚀 Building the Network", CYAN);
    animated_progress_bar("Building...", 200);
    thread::sleep(time::Duration::from_secs(1));
}

pub fn validating_network(){
    // 🔍 Validating the Network
    fancy_box("🔍 Validating the Network", YELLOW);
    animate_message("Validating...", &["🔍", "🔎"], 400);
    thread::sleep(time::Duration::from_secs(1));
}

pub fn network_running(){
    // Simulating network running
    fancy_box("🌐 Network Running", GREEN);
    animate_message("Network is running...", &["🔄", "🌀"], 500);
    thread::sleep(time::Duration::from_secs(2));
}

pub fn network_valid(){
    fancy_box("✅ Network Status: VALID", GREEN);
}

pub fn network_not_valid(){
    fancy_box("❌ Network Status: FAILED", RED);
}

pub fn network_stopped(){
    fancy_box("⚠️  Network Stopped", RED);
}