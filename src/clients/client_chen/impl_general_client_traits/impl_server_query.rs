use crate::clients::client_chen::prelude::*;
use crate::clients::client_chen::{ClientChen, Sending, ServerQuery};
use crate::clients::client_chen::general_client_traits::*;

impl ServerQuery for ClientChen{

    fn ask_server_type(&mut self, server_id: ServerId) {
        self.update_servers();
        if self.communication.servers.contains(&server_id) {
            self.send_query(server_id, Query::AskType);
        }
    }

    fn ask_list_files(&mut self, server_id: ServerId) {
        self.update_servers();
        if self.communication.servers.contains(&server_id) {
            self.send_query(server_id, Query::AskListFiles);
            println!("|WEB| CLIENT [{}] SENT QUERY ASK FILE LIST TO SERVER [{}]", self.metadata.node_id, server_id);
        }
    }

    fn ask_file(&mut self, server_id: ServerId, file_ref: String) {
        self.update_servers();
        if self.communication.servers.contains(&server_id) {
            self.send_query(server_id, Query::AskFile(file_ref));
        }
    }

    fn ask_media(&mut self, media_ref: String) {
        let media_servers = self.get_media_servers_from_topology();
        for server in media_servers{
            self.send_query(server, Query::AskMedia(media_ref.clone()));
        }
    }
}