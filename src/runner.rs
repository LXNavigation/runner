use crate::app_config::{AppConfig, AppMode};
use crate::config::Config;
use async_std::task;
use async_std::task::JoinHandle;
use futures::future::join_all;
use std::io::stdin;

pub fn run(config: String) {
    let config = match Config::create(config) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error parsing config file: {:?}", err);
            return;
        }
    };
    ensure_error_path(&config);

    let mut handles = Vec::new();
    for app in config.apps {
        if let Some(handle) = execute_app(app, config.crash_path.clone()) {
            handles.push(handle);
        }
    }
    task::block_on(join_all(handles));
    println!("All jobs finished, press enter to quit.");
    let mut s = String::new();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a valid string");
}

fn ensure_error_path(config: &Config) {
    std::fs::create_dir_all(&config.crash_path).expect("Could not create crash path, aborting...");
}

fn execute_app(app: AppConfig, error_path: String) -> Option<JoinHandle<()>> {
    match app.mode {
        AppMode::RunOnce => return Some(run_once(app, error_path)),
        AppMode::RunOnceAndWait => run_once_and_wait(app, error_path),
        AppMode::RunUntilSuccess => return Some(run_until_success(app, error_path)),
        AppMode::RunUntilSuccessAndWait => run_until_success_and_wait(app, error_path),
        AppMode::KeepAlive => return Some(run_keep_alive(app, error_path)),
    };
    None
}

fn run_once(app: AppConfig, error_path: String) -> JoinHandle<()> {
    task::spawn(async move {
        let _ = crate::run_command::run_command(app, error_path).await;
    })
}

fn run_once_and_wait(app: AppConfig, error_path: String) {
    task::block_on(async move {
        let _ = crate::run_command::run_command(app, error_path).await;
    });
}

fn run_until_success(app: AppConfig, error_path: String) -> JoinHandle<()> {
    task::spawn(async move {
        loop {
            if let Ok(()) = crate::run_command::run_command(app.clone(), error_path.clone()).await {
                return;
            }
        }
    })
}

pub(crate) fn run_until_success_and_wait(app: AppConfig, error_path: String) {
    task::block_on(async move {
        loop {
            if let Ok(()) = crate::run_command::run_command(app.clone(), error_path.clone()).await {
                return;
            }
        }
    });
}

pub(crate) fn run_keep_alive(app: AppConfig, error_path: String) -> JoinHandle<()> {
    task::spawn(async move {
        loop {
            let _ = crate::run_command::run_command(app.clone(), error_path.clone()).await;
        }
    })
}
