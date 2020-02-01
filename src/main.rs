extern crate  simplelog;

use simplelog::*;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web, Result};
use serde_json;
use std::{env, fs, io};
use std::path::Path;
use serde::{Deserialize, Serialize};
use log::{info, warn};
use std::process::{Command, Output, exit};
use actix_web::web::Data;

static LOGGER: &str = "hooker";

#[derive(Serialize, Deserialize)]
struct ConfigEntry {
    command: String,
    end_point: String,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hooker!")
}

async fn process_webook(path: web::Path<String>, config: web::Data<Vec<ConfigEntry>>) -> Result<String>  {
    let path = path.into_inner();
    for webhook in config.iter() {
        if webhook.end_point == path {
            let result = match run_command(&webhook.command) {
                Ok(result) => result,
                Err(_) => {
                    warn!(target: LOGGER, "Unable to run command for path {}", webhook.end_point);
                    return Ok(format!("Unable to run command!"))
                }
            };
            info!(target: LOGGER, "Command {} run for path {}", webhook.command, webhook.end_point);
            return Ok(format!("{}", result.status))
        }
    }

    warn!(target: LOGGER, "Webhook not found for path {}", path);
    Ok(format!("Webhook not found in config..."))
}

fn run_command(command: &String) -> Result<Output, io::Error>{
    let mut args = vec!["-c"];
    args.push(command.as_str());

    Command::new("/bin/sh")
        .args(args)
        .output()
}

fn parse_args(args: &[String]) -> (&str, &str, &str) {

    let ip = &args[1];
    let port = &args[2];
    let config_dir = &args[3];

    (ip, port, config_dir)
}

fn read_config(dir: &Path) -> Result<Vec<ConfigEntry>, io::Error>{
    let mut config: Vec<ConfigEntry> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let ext = match path.extension() {
            None => "",
            Some(ext) => ext.to_str().unwrap()
        };

        if path.is_file() && ext == "json"{
            let config_entry = fs::read_to_string(path)?;
            let config_entry:ConfigEntry = serde_json::from_str(&config_entry)?;

            config.push(config_entry);
        }


    }

    Ok(config)
}


async fn start_server(ip: &str, port: &str, config: Vec<ConfigEntry>) -> std::io::Result<()> {

    let data:Data<Vec<ConfigEntry>> = web::Data::new(config);
    HttpServer::new(move || {
        App::new()
            .service(index)
            .route("/{path}/", web::post().to(process_webook))
            .app_data(data.clone())
    })
        .bind(format!("{}:{}", ip, port))?
        .run()
        .await

}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("{}: [ip address] [port] [config path]", args[0]);
        return Ok(());
    }

    let (ip, port, config_path) = parse_args(&args);

    match CombinedLogger::init(
        vec![
            SimpleLogger::new(LevelFilter::Info, Config::default())
        ]
    ) {
        Err(_) => println!("Unable to start logging"),
        _ => ()
    }


    let config_path = Path::new(config_path);

    let config = match read_config(config_path) {
        Ok(config) => config,
        Err(e) => {
            warn!(target: LOGGER, "Unable to parse config directory: {}", e.to_string());
            exit(-1);
        }
    };

    info!(target: LOGGER, "Starting Hooker on {}:{}", ip, port);
    start_server(ip, port, config).await
}