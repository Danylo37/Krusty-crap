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

        // Validate each drone: check for duplicate IDs and individual drone fields.
        for drone in self.drones {
            if !seen_ids.insert(drone.id) {
                return Err(format!("Duplicate ID detected: {}", drone.id));
            }
            self.is_valid_drone(drone)?;
        }

        // Validate each client.
        for client in self.clients {
            if !seen_ids.insert(client.id) {
                return Err(format!("Duplicate ID detected: {}", client.id));
            }
            self.is_valid_client(client)?;
        }

        // Validate each server.
        for server in self.servers {
            if !seen_ids.insert(server.id) {
                return Err(format!("Duplicate ID detected: {}", server.id));
            }
            self.is_valid_server(server)?;
        }

        // Check that all drone-to-drone connections are bidirectional.
        self.check_bidirectional()?;

        // Check that the drone network is fully connected.
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

    /// Checks that every drone-to-drone connection is bidirectional.
    ///
    /// For each drone, if it lists a connection to another drone (i.e. a neighbor that is also defined as a drone),
    /// then the neighbor must also list the original drone in its `connected_node_ids`.
    fn check_bidirectional(&self) -> Result<(), String> {
        // Build a map of drone IDs to Drone objects for quick lookup.
        let drone_map: HashMap<DroneId, &Drone> = self.drones.iter().map(|d| (d.id, d)).collect();

        // Iterate over each drone and its connections.
        for drone in self.drones {
            for &neighbor in &drone.connected_node_ids {
                // Only check bidirectionality if the neighbor is also a drone in the configuration.
                if let Some(neighbor_drone) = drone_map.get(&neighbor) {
                    // If the neighbor drone does not have a reciprocal connection, report an error.
                    if !neighbor_drone.connected_node_ids.contains(&drone.id) {
                        return Err(format!(
                            "Drone {} is not bidirectionally connected with drone {}",
                            drone.id, neighbor
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Checks if the drone network is fully connected.
    ///
    /// This method constructs a symmetric graph from the drone connections.
    /// Since bidirectionality has been validated above, we add the edges in both directions
    /// for drones that exist in the configuration.
    fn is_network_connected(&self) -> bool {
        let mut graph: HashMap<DroneId, HashSet<DroneId>> = HashMap::new();

        // Build the graph using only drones.
        for drone in self.drones {
            graph.entry(drone.id).or_default();
            // Add edges only for neighbors that are also drones.
            for &node in &drone.connected_node_ids {
                if self.drones.iter().any(|d| d.id == node) {
                    graph.entry(node).or_default().insert(drone.id);
                    graph.entry(drone.id).or_default().insert(node);
                }
            }
        }

        if graph.is_empty() {
            return false;
        }

        // Use DFS to check connectivity.
        let start = *graph.keys().next().unwrap();
        let mut visited = HashSet::new();
        self.dfs(start, &graph, &mut visited);

        // The network is fully connected if all nodes in the graph are visited.
        visited.len() == graph.len()
    }

    /// Depth-first search helper function.
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
