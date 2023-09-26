use log::error;
use std::path::PathBuf;

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct Client {
    pub name: String,
    pub short_name: String,
}

impl Client {
    fn get_clients(clients_path: PathBuf) -> Result<Vec<Client>, String> {
        let f = match std::fs::File::open(clients_path) {
            Ok(f) => f,
            Err(e) => {
                let message = format!("Failed to get client list: {}", e);
                error!("{}", e);
                return Err(message);
            }
        };

        let clients: Vec<Client> = match serde_yaml::from_reader(f) {
            Ok(c) => c,
            Err(e) => {
                let message = format!("Failed to get client list: {}", e);
                error!("{}", e);
                return Err(message);
            }
        };

        return Ok(clients);
    }
}
