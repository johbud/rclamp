use log::error;
use log::info;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;

use crate::helpers::sanitize_string;

/// When creating a project, the user can choose from a list of clients names, which will inserted into the project name.
/// Client consists of a full name, which appears in the UI, and a short sanitized name used for the actual project name.
#[derive(Clone, serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub struct Client {
    pub name: String,
    pub short_name: String,
}

impl Client {
    /// Open the file containing the list of clients, read only.
    fn open_clients_file(clients_path: PathBuf) -> Result<File, String> {
        info!(
            "Attempting to open: {}",
            clients_path.clone().to_string_lossy()
        );
        match std::fs::File::open(clients_path.clone()) {
            Ok(f) => return Ok(f),
            Err(e) => {
                let message = format!(
                    "Failed to open file {}: {}",
                    clients_path.clone().to_string_lossy(),
                    e
                );
                error!("{}", message);
                return Err(message);
            }
        };
    }

    /// Parses the file, using serde_yaml, into a Vec of Client structs.
    pub fn get_clients(clients_path: PathBuf) -> Result<Vec<Client>, String> {
        let f = match Client::open_clients_file(clients_path) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        let clients: Vec<Client> = match serde_yaml::from_reader(f) {
            Ok(c) => c,
            Err(e) => {
                let message = format!("Failed to get client list: {}", e);
                error!("{}", message);
                return Err(message);
            }
        };

        return Ok(clients);
    }

    /// Creates and sanitizes a client struct, then checks for duplicates in the current client vec, then appends the new client. Finally writes to file.
    pub fn add_client(
        name: &String,
        short_name: &String,
        clients_path: &PathBuf,
    ) -> Result<(), String> {
        // Read in clients list.
        let mut clients = match Client::get_clients(clients_path.to_owned()) {
            Ok(c) => c,
            Err(e) => {
                return Err(e);
            }
        };

        // Sanitize the short name.
        let sanitized_short_name = sanitize_string(short_name.to_owned());

        // Create new client struct and check for duplicate clients, then push to vec.
        let new_client = Client {
            name: name.to_string(),
            short_name: sanitized_short_name,
        };
        if Client::check_for_duplicate_clients(&clients, &new_client) {
            return Err(String::from("Client with same name already exists."));
        }

        clients.push(new_client);

        match Client::write_clients_to_file(clients, clients_path.to_owned()) {
            Ok(_o) => (),
            Err(e) => {
                return Err(e);
            }
        }
        Ok(())
    }

    /// Writes a list of clients to a file using serde_yaml.
    fn write_clients_to_file(clients: Vec<Client>, path: PathBuf) -> Result<(), String> {
        info!("Writing: {:#?}", clients);
        // Open the clients file for writing.
        let f = match OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.clone())
        {
            Ok(f) => f,
            Err(e) => return Err(e.to_string()),
        };

        // Overwrite the current clients list file with the modified list.
        match serde_yaml::to_writer(f, &clients) {
            Ok(_o) => info!("Wrote to file."),
            Err(e) => {
                let message = format!("Failed to write file {}: {}", path.to_string_lossy(), e);
                error!("{}", message);
                return Err(message);
            }
        }
        Ok(())
    }

    /// Returns true if duplicate is found, otherwise false.
    fn check_for_duplicate_clients(client_list: &Vec<Client>, new_client: &Client) -> bool {
        for c in client_list.iter() {
            if c.name == new_client.name || c.short_name == new_client.short_name {
                return true;
            }
        }
        false
    }

    /// Takes a client struct, finds and removes clients with identical name in the file at eh supplied path, and writes to file.
    pub fn remove_client(client: &Client, clients_path: &PathBuf) -> Result<(), String> {
        info!("Attempting to remove: {}", client.name);
        // Get a current list of clients.
        let clients = match Client::get_clients(clients_path.to_owned()) {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        // Remove the selected client by filtering.
        let clients_filtered: Vec<Client> = clients
            .iter()
            .filter(|c| c.name != client.name)
            .map(|c| c.to_owned())
            .collect();

        info!("Filtered list: {:#?}", clients_filtered);

        // Write to file.
        match Client::write_clients_to_file(clients_filtered, clients_path.to_owned()) {
            Ok(_o) => (),
            Err(e) => {
                let message = format!("Failed remove client: {}", e);
                error!("{}", message);
                return Err(message);
            }
        }

        Ok(())
    }
}
