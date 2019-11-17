use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use structopt::StructOpt;
use crate::config::Config;

mod config;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "parametrik_cli",
    about = "Command-line interface for Parametrik"
)]
struct Opt {
    #[structopt(short, long, default_value = "http://localhost:3001")]
    url: String,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    Login {
        #[structopt(short)]
        email: Option<String>,
        #[structopt(short)]
        password: Option<String>,
    },
    Register,
}

fn main() {
    let opts = Opt::from_args();
    let config = Config::new();
    match opts.cmd {
        Command::Login { email, password } => {
            run_login_command(&email, &password, &opts.url, &config).unwrap()
        }
        Command::Register => run_register_command(&opts.url).unwrap(),
    };
}

#[derive(Deserialize)]
struct LoginResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
}

fn run_login_command(
    maybe_email: &Option<String>,
    maybe_password: &Option<String>,
    url: &String,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let email = match maybe_email {
        None => dialoguer::Input::<String>::new()
            .with_prompt("Email")
            .interact()?,
        Some(e) => e.to_owned(),
    };
    let password = match maybe_password {
        None => dialoguer::PasswordInput::new()
            .with_prompt("Password")
            .interact()?,
        Some(p) => p.to_owned(),
    };

    let body = json!({
        "email": email,
        "password": password,
    });

    let user_tokens_url = format!("{}/v1/user_tokens", url);
    let access_token: reqwest::Result<LoginResponse> = reqwest::Client::new()
        .post(&user_tokens_url)
        .json(&body)
        .send()?
        .json();

    match access_token {
        Ok(LoginResponse { access_token }) => {
            match &config.path {
                None => println!("{}", &access_token),
                Some(config_dir) => {
                    let mut auth_path = config_dir.clone();
                    auth_path.push("access_token");
                    std::fs::write(auth_path, &access_token)?;
                },
            }
            Ok(())
        },
        _ => {
            eprintln!("Something went wrong");
            ::std::process::exit(1);
        }
    }
}

fn run_register_command(url: &String) -> Result<(), Box<dyn Error>> {
    let name = dialoguer::Input::<String>::new()
        .with_prompt("Full name")
        .interact()?;
    let email = dialoguer::Input::<String>::new()
        .with_prompt("Email")
        .interact()?;
    let password = dialoguer::PasswordInput::new()
        .with_prompt("Password")
        .with_confirmation("Please re-enter your password", "Passwords do not match")
        .interact()?;

    let body = json!({
        "name": name,
        "email": email,
        "password": password,
    });

    let users_url = format!("{}/v1/users", url);
    let response = reqwest::Client::new().post(&users_url).json(&body).send()?;

    match response.status() {
        StatusCode::CREATED => {
            println!("User registered");
            Ok(())
        }
        StatusCode::CONFLICT => {
            eprintln!("You are already registered. Use `para login` instead.");
            ::std::process::exit(1);
        }
        _ => {
            eprintln!("Something went wrong");
            ::std::process::exit(1);
        }
    }
}
