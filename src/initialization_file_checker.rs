use std::collections::{HashMap, HashSet};
use wg_2024::config::{Client, Config, Drone, Server};
use crate::general_use::DroneId;

pub struct InitializationFileChecker<'a> {
    drones: &'a Vec<Drone>,
    clients: &'a Vec<Client>,
    servers: &'a Vec<Server>,
}

impl<'a> InitializationFileChecker<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            drones: &config.drone,
            clients: &config.client,
            servers: &config.server,
        }
    }

    pub fn check(&self) -> Result<(), String> {
        let mut seen_ids = HashSet::new();

        for drone in self.drones {
            if !seen_ids.insert(drone.id) {
                return Err(format!("Duplicate ID detected: {}", drone.id));
            }
            self.is_valid_drone(drone)?;
        }

        for client in self.clients {
            if !seen_ids.insert(client.id) {
                return Err(format!("Duplicate ID detected: {}", client.id));
            }
            self.is_valid_client(client)?;
        }

        for server in self.servers {
            if !seen_ids.insert(server.id) {
                return Err(format!("Duplicate ID detected: {}", server.id));
            }
            self.is_valid_server(server)?;
        }

        if !self.is_network_connected() {
            return Err("The network is not fully connected after removing clients and servers.".to_string());
        }

        Ok(())
    }

    fn is_valid_drone(&self, drone: &Drone) -> Result<(), String> {
        if !(0.0..=1.0).contains(&drone.pdr) {
            return Err(format!("Drone {} has invalid PDR: {}", drone.id, drone.pdr));
        }

        if drone.connected_node_ids.contains(&drone.id) {
            return Err(format!("Drone {} is connected to itself", drone.id));
        }

        if drone.connected_node_ids.len() != drone.connected_node_ids.iter().collect::<HashSet<_>>().len() {
            return Err(format!("Drone {} has duplicate connections", drone.id));
        }

        Ok(())
    }

    fn is_valid_client(&self, client: &Client) -> Result<(), String> {
        let num_of_connections = client.connected_drone_ids.len();

        if num_of_connections == 0 || num_of_connections > 2 {
            return Err(format!(
                "Client {} has an invalid number of connected drones: {}",
                client.id, num_of_connections
            ));
        }

        if client.connected_drone_ids.contains(&client.id) {
            return Err(format!("Client {} is connected to itself", client.id));
        }

        Ok(())
    }

    fn is_valid_server(&self, server: &Server) -> Result<(), String> {
        let num_of_connections = server.connected_drone_ids.len();

        if num_of_connections < 2 {
            return Err(format!(
                "Server {} must have at least 2 connected drones, found: {}",
                server.id, num_of_connections
            ));
        }

        if server.connected_drone_ids.contains(&server.id) {
            return Err(format!("Server {} is connected to itself", server.id));
        }

        Ok(())
    }

    fn is_network_connected(&self) -> bool {
        let mut graph: HashMap<DroneId, HashSet<DroneId>> = HashMap::new();

        // Add only drones to the graph
        for drone in self.drones {
            graph.entry(drone.id).or_default();
            for &node in &drone.connected_node_ids {
                graph.entry(node).or_default().insert(drone.id);
                graph.entry(drone.id).or_default().insert(node);
            }
        }

        if graph.is_empty() {
            return false;
        }

        // DFS for checking connectivity
        let start = *graph.keys().next().unwrap();
        let mut visited = HashSet::new();
        self.dfs(start, &graph, &mut visited);

        visited.len() == graph.len()
    }

    fn dfs(&self, node: DroneId, graph: &HashMap<DroneId, HashSet<DroneId>>, visited: &mut HashSet<DroneId>) {
        if visited.insert(node) {
            if let Some(neighbors) = graph.get(&node) {
                for &neighbor in neighbors {
                    self.dfs(neighbor, graph, visited);
                }
            }
        }
    }
}