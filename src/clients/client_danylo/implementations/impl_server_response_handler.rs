use log::{debug, error, info};
use crate::general_use::{ClientId, Message, Response, ServerId, ServerType};
use super::{ServerResponseHandler, ChatClientDanylo};

impl ServerResponseHandler for ChatClientDanylo {
    /// ###### Handles the server response.
    /// Processes the server response based on its type and takes appropriate actions.
    fn handle_server_response(&mut self, response: Option<Response>, server_id: ServerId) {
        debug!("Client {}: Handling response from server {}: {:?}", self.id, server_id, response);

        if let Some(response) = response {
            match response {
                Response::ServerType(server_type) => {
                    self.handle_server_type(server_id, server_type);
                },
                Response::ClientRegistered => {
                    self.handle_client_registered(server_id);
                }
                Response::ListClients(clients) => {
                    self.handle_clients_list(server_id, clients);
                }
                Response::MessageFrom(from, message) => {
                    self.handle_message_from(from, message);
                }
                Response::Err(error) =>
                    error!("Client {}: Error received from server {}: {:?}", self.id, server_id, error),
                _ => {}
            }
        }
    }

    /// ###### Handles the server type response.
    /// Updates the server type in the `servers` map and
    /// sets the registration status if the server is of type `Communication`.
    fn handle_server_type(&mut self, server_id: ServerId, server_type: ServerType) {
        info!("Client {}: Server type received successfully.", self.id);

        // Insert the server type into the servers map.
        self.servers.insert(server_id, server_type);

        // If the server is of type Communication, set the registration status to false.
        if !self.is_registered.contains_key(&server_id) && server_type == ServerType::Communication {
            self.is_registered.insert(server_id, false);
        }
    }

    /// ###### Handles the client registration response.
    /// Updates the registration status for the specified server.
    fn handle_client_registered(&mut self, server_id: ServerId) {
        info!("Client {}: Client registered successfully.", self.id);

        self.is_registered.insert(server_id, true);
    }

    /// ###### Handles the list of clients received from the server.
    /// Updates the list of available clients.
    fn handle_clients_list(&mut self, server_id: ServerId, mut clients: Vec<ClientId>) {
        info!("Client {}: List of clients received successfully.", self.id);

        // Remove self id from the clients list if it exists
        if clients.contains(&self.id) {
            clients.retain(|&client_id| client_id != self.id);
        }

        self.clients.insert(server_id, clients);
    }

    /// ###### Handles the message received from another client.
    /// Adds the message to the chat history with the sender.
    fn handle_message_from(&mut self, from: ClientId, message: Message) {
        info!("Client {}: New message from {}: {:?}", self.id, from, &message);

        let chat = self.chats.entry(from).or_insert_with(Vec::new);
        chat.push((from, message));
    }
}