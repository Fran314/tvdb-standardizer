#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Target {
    pub label: String,
    pub path: String,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub tvdb_auth_token: String,
    pub storage_path: String,
    pub targets: Vec<Target>,
}

pub fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = (dirs::config_dir().ok_or("couldn't find config dir")?)
        .join("tvdb-standardizer")
        .join("config.toml");
    let mut file = std::fs::File::open(config_path)?;
    let config_contents = {
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut file, &mut contents)?;
        contents
    };

    let config: Config = toml::from_str(&config_contents)?;
    Ok(config)
}
