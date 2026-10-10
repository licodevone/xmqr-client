mod cli;
mod credentials;
mod output;
mod payload;
mod session;
mod signals;
mod tls;

#[tokio::main]
async fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.len() == 1 && matches!(arguments[0].as_str(), "--help" | "-h") {
        print!("{}", cli::USAGE);
        return;
    }

    if arguments.len() == 1 && arguments[0] == "--version" {
        println!("mqtt-client {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let cli = match cli::parse(arguments) {
        Ok(cli) => cli,
        Err(message) => {
            eprintln!("Erro: {message}\n\n{}", cli::USAGE);
            std::process::exit(2);
        }
    };
    let publication = if let cli::Mode::Pub { message, .. } = &cli.mode {
        let Ok(mut input) = payload::Input::start(message.clone(), cli.line_mode) else {
            eprintln!("Erro: nao foi possivel preparar entrada de payload");
            std::process::exit(2);
        };
        let first = tokio::select! {
            result = input.next() => match result {
                Ok(Some(bytes)) => bytes,
                Ok(None) => return, // Empty line input does not connect or publish.
                Err(error) => { eprintln!("Erro: {error}"); std::process::exit(2); }
            },
            signal = signals::shutdown() => {
                if signal.is_err() { std::process::exit(1); }
                eprintln!("Cancelado por sinal de encerramento");
                return;
            }
        };
        Some(payload::Publication { input, first })
    } else {
        None
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
    if let Err(error) = session::run(cli, password, publication).await {
        eprintln!("Erro: {error}");
        std::process::exit(1);
    }
}
