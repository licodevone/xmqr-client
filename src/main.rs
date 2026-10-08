mod cli;
mod credentials;
mod output;
mod session;
mod tls;

#[tokio::main]
async fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.len() == 1 && matches!(arguments[0].as_str(), "--help" | "-h") {
        print!("{}", cli::USAGE);
        return;
    }

    let cli = match cli::parse(arguments) {
        Ok(cli) => cli,
        Err(message) => {
            eprintln!("Erro: {message}\n\n{}", cli::USAGE);
            std::process::exit(2);
        }
    };
    // Open-lab is deliberately anonymous; secure mode prompts before opening
    // the network connection and never places the password in argv.
    let password = if cli.transport_mode == cli::TransportMode::OpenLab {
        None
    } else {
        let password_result = if let Some(ref path) = cli.password_file {
            credentials::read_password_file(path)
        } else {
            rpassword::prompt_password("Senha MQTT: ")
        };
        let Ok(password) = password_result else {
            eprintln!("Erro: nao foi possivel ler a senha MQTT com seguranca");
            std::process::exit(2);
        };
        if let Err(message) = cli::validate_password(&password) {
            eprintln!("Erro: {message}");
            std::process::exit(2);
        }
        Some(password)
    };
    if let Err(error) = session::run(cli, password).await {
        eprintln!("Erro: {error}");
        std::process::exit(1);
    }
}
