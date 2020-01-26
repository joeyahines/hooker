use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web, Result};
use serde_json;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::exit;
use serde::{Deserialize, Serialize};
use log::{info, warn};
use std::process::{Command, Output};
use std::borrow::BorrowMut;

#[derive(Serialize, Deserialize)]
struct ConfigEntry {
    command: String,
    webhook: serde_json::Value,

}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hooker!")
}

#[post("/hooker")]
async fn hooker(config: web::Data<Vec<ConfigEntry>>, json: web::Json<serde_json::Value>) -> Result<String>  {
    if json.is_object() {
        let json = json.as_object().unwrap();

        for (key, value) in json.iter() {
            for config_entry in config.iter() {
                let mut matches = true;
                let webhook = config_entry.webhook.as_object().unwrap();

                for (config_key, config_value) in webhook {
                    if config_key == key {
                        if config_value != value {
                            matches = false;
                        }
                    }
                }

                if matches {
                    println!("{}", "Match found");
                    run_command(&config_entry.command);

                }
                else {
                    println!("{}", "Match not found");
                }
            }
        }
    }

    Ok(format!("test"))
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

        if path.is_file() {
            let config_entry = fs::read_to_string(path)?;
            let config_entry:ConfigEntry = serde_json::from_str(&config_entry)?;

            config.push(config_entry);

        }


    }

    Ok(config)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let (ip, port, config_path) = parse_args(&args);

    println!("{},{},{}", ip, port, config_path);
    let config_path = Path::new(config_path);


    let config = match read_config(config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("Unable to parse config directory: {}", e.to_string());
            exit(-1);
        }
    };

    let config = web::Data::new(config);

    HttpServer::new(move|| {
        App::new()
            .app_data(config.clone())
            .service(index)
            .service(hooker)
    })
        .bind(format!("{}:{}", ip, port))?
        .run()
        .await
}