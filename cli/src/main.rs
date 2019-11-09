use std::error::Error;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "parametrik_cli",
    about = "Command-line interface for Parametrik"
)]
struct Opt {
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
        Command::Login { email, password } => run_login_command(&email, &password).unwrap(),
        Command::Register => run_register_command().unwrap(),
    };
}

fn run_login_command(
    maybe_email: &Option<String>,
    maybe_password: &Option<String>,
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

    println!("email: {:?}, password: {:?}", email, password);
    Ok(())
}

fn run_register_command() -> Result<(), Box<dyn Error>> {
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

    println!(
        "name: {:?}, email: {:?}, password: {:?}",
        name, email, password
    );
    Ok(())
}
