mod cli;
mod config;
mod manager;

use std::{env::current_dir, process::exit};

use anyhow::{Context, Error};
use clap::Parser;
use cli::{Cli, SubCommand};
use dir::home_dir;
use manager::AppManager;

fn print_chained_error(err: Error) {
    for e in err.chain() {
        println!("[Err] {}", e);
    }
}

fn main() {
    let config_dir_path = match home_dir().context("Failed to get the home directory.") {
        Ok(dir) => dir,
        Err(err) => {
            print_chained_error(err);
            exit(-1);
        }
    }
    .join(".config")
    .join("pwnenv");

    let mut app_manager = AppManager::new(&config_dir_path);
    if let Err(err) = app_manager
        .setup_minimum_requirements()
        .context("Failed to init the app.")
    {
        print_chained_error(err);
        exit(-1)
    };

    let cli = Cli::parse();

    match cli.sub_command {
        SubCommand::Init => {
            let current_dir_path =
                match current_dir().context("Failed to get the current directory path.") {
                    Ok(dir) => dir,
                    Err(err) => {
                        print_chained_error(err);
                        exit(-1);
                    }
                };

            app_manager.init(&current_dir_path).unwrap();
        }
        SubCommand::Enter => {
            app_manager.enter().unwrap();
        }
        SubCommand::Kill => {
            app_manager.kill().unwrap();
        }
    }
}
