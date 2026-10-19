//! C147: configuração v2 e isolamento de execução por mapeamento.
use std::{fs, io, path::PathBuf};

use serde::Deserialize;
use tokio::task::JoinSet;

use super::{
    AppResult, Config, Endpoint, Route, SourceProtocol, default_retries, invalid, prefixes_overlap,
    tls, valid_id, valid_prefix, validate,
};

const MAX_ROUTES: usize = 8;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub id: String,
    pub source_prefix: String,
    pub target_prefix: String,
    #[serde(default)]
    pub qos: u8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiConfig {
    version: u8,
    #[serde(default)]
    source_protocol: SourceProtocol,
    source: Endpoint,
    destination: Endpoint,
    routes: Vec<Mapping>,
    topology_file: PathBuf,
    state_dir: Option<PathBuf>,
    #[serde(default = "default_retries")]
    max_retries: u8,
}

pub enum Loaded {
    Single(Config),
    Multi(MultiConfig),
}

pub fn private_directory(path: &std::path::Path) -> AppResult<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !path.is_absolute() || !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(invalid("diretorio de estado deve ser absoluto, real e privado").into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(invalid("diretorio de estado exige modo 0700 ou mais restrito").into());
        }
    }
    Ok(())
}

impl MultiConfig {
    fn shape(&self) -> AppResult<()> {
        if self.version != 2 || self.routes.is_empty() || self.routes.len() > MAX_ROUTES {
            return Err(invalid("configuracao v2 exige de uma a oito rotas").into());
        }
        for (index, route) in self.routes.iter().enumerate() {
            if !valid_id(&route.id)
                || route.id.len() > 32
                || !valid_prefix(&route.source_prefix)
                || !valid_prefix(&route.target_prefix)
                || route.qos > 2
                || self.routes[..index].iter().any(|previous| {
                    previous.id == route.id
                        || prefixes_overlap(&previous.source_prefix, &route.source_prefix)
                })
            {
                return Err(invalid("rota duplicada, ambigua ou invalida").into());
            }
            for endpoint in [&self.source, &self.destination] {
                if endpoint.client_id.len() + 1 + route.id.len() > 128 {
                    return Err(invalid("identidade MQTT derivada acima de 128 bytes").into());
                }
            }
        }
        if self.routes.iter().any(|route| route.qos == 2) && self.state_dir.is_none() {
            return Err(invalid("rotas QoS 2 exigem state_dir privado").into());
        }
        if let Some(directory) = &self.state_dir {
            private_directory(directory)?;
        }
        Ok(())
    }

    fn workers(&self) -> AppResult<Vec<Config>> {
        self.shape()?;
        let mut result = Vec::with_capacity(self.routes.len());
        for mapping in &self.routes {
            let mut source = self.source.clone();
            let mut destination = self.destination.clone();
            source.client_id = format!("{}-{}", source.client_id, mapping.id);
            destination.client_id = format!("{}-{}", destination.client_id, mapping.id);
            let journal_dir = if mapping.qos == 2 {
                let path = self.state_dir.as_ref().unwrap().join(&mapping.id);
                match fs::symlink_metadata(&path) {
                    Ok(_) => private_directory(&path)?,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
                Some(path)
            } else {
                None
            };
            let worker = Config {
                version: 1,
                source_protocol: self.source_protocol,
                source,
                destination,
                route: Route {
                    source_prefix: mapping.source_prefix.clone(),
                    target_prefix: mapping.target_prefix.clone(),
                    qos: mapping.qos,
                },
                topology_file: Some(self.topology_file.clone()),
                link_id: Some(mapping.id.clone()),
                journal_dir,
                max_retries: self.max_retries,
            };
            validate(&worker)?;
            result.push(worker);
        }
        Ok(result)
    }

    fn prepare(self) -> AppResult<Vec<Config>> {
        let configs = self.workers()?;
        for config in &configs {
            if let Some(directory) = &config.journal_dir {
                let mut builder = fs::DirBuilder::new();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::DirBuilderExt;
                    builder.mode(0o700);
                }
                match builder.create(directory) {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(error.into()),
                }
                private_directory(directory)?;
            }
        }
        Ok(configs)
    }

    async fn run(self) -> AppResult<()> {
        let configs = tokio::task::spawn_blocking(move || self.prepare()).await??;
        let mut tasks = JoinSet::new();
        for config in configs {
            eprintln!(
                "Bridge: iniciando rota {}",
                config.link_id.as_deref().unwrap()
            );
            tasks.spawn(super::run(config));
        }
        while let Some(result) = tasks.join_next().await {
            result??;
        }
        Ok(())
    }
}

impl Loaded {
    pub fn route_count(&self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Multi(config) => config.routes.len(),
        }
    }

    pub async fn run(self) -> AppResult<()> {
        match self {
            Self::Single(config) => super::run(config).await,
            Self::Multi(config) => config.run().await,
        }
    }
}

pub fn load(bytes: &[u8], check_tls: bool) -> AppResult<Loaded> {
    let text = std::str::from_utf8(bytes)?;
    let document: toml::Value = toml::from_str(text)?;
    let loaded = match document.get("version").and_then(toml::Value::as_integer) {
        Some(1) => {
            let config: Config = toml::from_str(text)?;
            validate(&config)?;
            if check_tls {
                tls(&config.source)?;
                tls(&config.destination)?;
            }
            Loaded::Single(config)
        }
        Some(2) => {
            let config: MultiConfig = toml::from_str(text)?;
            config.workers()?;
            if check_tls {
                tls(&config.source)?;
                tls(&config.destination)?;
            }
            Loaded::Multi(config)
        }
        _ => return Err(invalid("versao de configuracao da bridge desconhecida").into()),
    };
    Ok(loaded)
}

#[cfg(test)]
mod tests {
    use super::{Mapping, MultiConfig};
    use crate::{Endpoint, SourceProtocol};

    fn config() -> MultiConfig {
        fn endpoint(client_id: &str, port: u16) -> Endpoint {
            Endpoint {
                broker_id: None,
                host: "127.0.0.1".into(),
                port,
                client_id: client_id.into(),
                username: "teste".into(),
                password_file: "/privado/senha".into(),
                ca: "/privado/ca".into(),
                cert: "/privado/certificado".into(),
                key: "/privado/chave".into(),
            }
        }
        MultiConfig {
            version: 2,
            source_protocol: SourceProtocol::Mqtt5,
            source: endpoint("origem", 8883),
            destination: endpoint("destino", 8884),
            routes: vec![Mapping {
                id: "temperatura".into(),
                source_prefix: "sensores/".into(),
                target_prefix: "espelho/".into(),
                qos: 1,
            }],
            topology_file: "/privado/topologia".into(),
            state_dir: None,
            max_retries: 8,
        }
    }

    #[test]
    fn rotas_recusam_sobreposicao_ids_duplicados_e_quantidades_invalidas() {
        let mut config = config();
        assert!(config.shape().is_ok());
        config.routes.push(Mapping {
            id: "umidade".into(),
            source_prefix: "sensores/sala/".into(),
            target_prefix: "outro/".into(),
            qos: 0,
        });
        assert!(config.shape().is_err());
        config.routes[1].source_prefix = "outros/".into();
        assert!(config.shape().is_ok());
        config.routes[1].id = "temperatura".into();
        assert!(config.shape().is_err());
        config.routes.clear();
        assert!(config.shape().is_err());
        for index in 0..9 {
            config.routes.push(Mapping {
                id: format!("rota-{index}"),
                source_prefix: format!("sensor-{index}/"),
                target_prefix: "espelho/".into(),
                qos: 0,
            });
        }
        assert!(config.shape().is_err());
        config.routes.pop();
        assert!(config.shape().is_ok());
    }

    #[test]
    fn estado_e_identidades_derivadas_respeitam_limites() {
        let mut config = config();
        config.source.client_id = "x".repeat(128);
        assert!(config.shape().is_err());
        config.source.client_id = "origem".into();
        config.routes[0].qos = 2;
        assert!(config.shape().is_err());
        let state = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        config.state_dir = Some(state.path().into());
        assert!(config.shape().is_ok());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o777)).unwrap();
            assert!(config.shape().is_err());
        }
    }
}
