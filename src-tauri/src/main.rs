#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use vscraper_lib::{Args, run};

fn main() {
    run(Args::parse())
}