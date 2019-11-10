use reqwest::StatusCode;
use serde_json::json;
use std::error::Error;
use structopt::StructOpt;

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
    match opts.cmd {
        Command::Login { email, password } => {
            run_login_command(&email, &password, &opts.url).unwrap()
        }
        Command::Register => run_register_command(&opts.url).unwrap(),
    };
}

fn run_login_command(
    maybe_email: &Option<String>,
    maybe_password: &Option<String>,
    url: &String,
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

    let user_tokens_url = format!("{}/v1/user_tokens", url);
    let response = reqwest::Client::new()
        .get(&user_tokens_url)
        .basic_auth(&email, Some(&password))
        .send()?;

    match response.status() {
        StatusCode::OK => Ok(()),
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
        _ => {
            eprintln!("Something went wrong");
            ::std::process::exit(1);
        }
    }
}
