// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Luis E. S. Pinheiro
// C27 contract reconstruction: newly authored implementation, not production source.
mod args;
mod engine;
mod format;
mod security;

#[tokio::main]
async fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.len() == 1 && (arguments[0] == "--help" || arguments[0] == "-h") {
        print!("{}", args::HELP);
        return;
    }
    let settings = match args::parse(arguments) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Erro: {error}\n{}", args::HELP);
            std::process::exit(2);
        }
    };
    let Ok(secret) = security::password(&settings) else {
        eprintln!("Erro: nao foi possivel ler a senha MQTT com seguranca");
        std::process::exit(2);
    };
    if let Err(error) = engine::start(&settings, secret).await {
        eprintln!("Erro: {error}");
        std::process::exit(1);
    }
}
