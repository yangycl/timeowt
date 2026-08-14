use std::env;
use std::process::{Command, exit};
use std::io::Write;
use std::thread;
use std::time::Duration;
use chrono::Local;

fn print_usage() {
    println!("Error: Invalid arguments.");
    println!("Usage:");
    println!("  One-time run: timeowt <hh> <mm> <script_name>.sh");
    println!("  Daily schedule via cron: timeowt -re <hh> <mm> <script_name>.sh");
    println!("Example: timeowt -re 14 30 /home/user/backup.sh");
}

fn add_to_cron(hour: u32, minute: u32, script_path: &str) {
    let abs_path = if script_path.starts_with('/') {
        script_path.to_string()
    } else {
        match env::current_dir() {
            Ok(mut p) => {
                p.push(script_path);
                p.to_string_lossy().to_string()
            }
            Err(_) => script_path.to_string(),
        }
    };

    let cron_job = format!("{} {} * * * /bin/bash {}\n", minute, hour, abs_path);

    println!("Registering daily task to system cron...");

    let output = Command::new("crontab").arg("-l").output();
    let mut current_cron = if output.is_ok() && output.as_ref().unwrap().status.success() {
        String::from_utf8_lossy(&output.unwrap().stdout).to_string()
    } else {
        String::new()
    };

    if current_cron.contains(&abs_path) {
        println!("Warning: The specified script is already present in crontab. Skipping.");
        return;
    }

    current_cron.push_str(&cron_job);

    let mut child = Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to execute crontab command");

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(current_cron.as_bytes()).expect("Failed to write to crontab");
    }

    let status = child.wait().expect("Failed to wait for crontab process");
    if status.success() {
        println!("Success: Configured daily execution via cron at {:02}:{:02}.", hour, minute);
        println!("Run 'crontab -l' to view your active schedules.");
    } else {
        println!("Error: Failed to write to crontab.");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 || args.len() > 5 {
        print_usage();
        exit(1);
    }

    let mut is_repeat = false;
    let hh_str: &str;
    let mm_str: &str;
    let script_path: &str;

    if args[1] == "-re" {
        if args.len() != 5 {
            print_usage();
            exit(1);
        }
        is_repeat = true;
        hh_str = &args[2];
        mm_str = &args[3];
        script_path = &args[4];
    } else {
        if args.len() != 4 {
            print_usage();
            exit(1);
        }
        hh_str = &args[1];
        mm_str = &args[2];
        script_path = &args[3];
    }

    let target_hour: u32 = hh_str.parse().unwrap_or_else(|_| {
        println!("Error: Invalid hour format. Must be an integer (0-23).");
        exit(1);
    });

    let target_minute: u32 = mm_str.parse().unwrap_or_else(|_| {
        println!("Error: Invalid minute format. Must be an integer (0-59).");
        exit(1);
    });

    if target_hour > 23 || target_minute > 59 {
        println!("Error: Time out of bounds. Hour: 0-23, Minute: 0-59.");
        exit(1);
    }

    if is_repeat {
        add_to_cron(target_hour, target_minute, script_path);
    } else {
        println!("Mode: One-time run. Waiting until {:02}:{:02} to execute...", target_hour, target_minute);
        
        loop {
            let now = Local::now();
            let target_time = chrono::NaiveTime::from_hms_opt(target_hour, target_minute, 0).unwrap();
            let current_time = now.time();
            
            let duration_to_wait = if current_time < target_time {
                (target_time - current_time).num_seconds()
            } else {
                let seconds_to_end_of_day = (chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap() - current_time).num_seconds() + 1;
                let seconds_next_day = (target_time - chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()).num_seconds();
                seconds_to_end_of_day + seconds_next_day
            };

            println!("Time remaining: {} seconds.", duration_to_wait);
            thread::sleep(Duration::from_secs(duration_to_wait as u64));

            println!("Executing script...");
            let _ = Command::new("bash").arg(script_path).status();
            println!("Task finished.");
            break;
        }
    }
}
