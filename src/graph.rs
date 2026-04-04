use crate::errors::GraphCreationError;
use std::{collections::HashMap, sync::Arc};

#[derive(Eq, Hash, PartialEq)]
pub struct Node {
    id: usize,
}
pub struct Edge {
    left: Arc<Node>,
    right: Arc<Node>,
    weight: f32,
}

pub struct Graph {
    nodes: Vec<Arc<Node>>,
    edges: Vec<Arc<Edge>>,
    real_ids: Vec<usize>,
    connections: Vec<Vec<Arc<Edge>>>,
}

impl Graph {
    pub fn new(
        nodes: &[usize],
        edges: &[(usize, usize, f32)],
    ) -> Result<Graph, GraphCreationError> {
        if !Graph::verify_node_edges_correctness(nodes, edges) {
            return Err(GraphCreationError);
        }

        let mut n_nodes: Vec<Arc<Node>> = Vec::with_capacity(nodes.len());
        let mut real_ids: Vec<usize> = Vec::with_capacity(nodes.len());
        let mut real_ids_to_local_ids: HashMap<usize, usize> = HashMap::with_capacity(nodes.len());

        let mut n_edges: Vec<Arc<Edge>> = Vec::with_capacity(edges.len());
        let mut connections: Vec<Vec<Arc<Edge>>> = Vec::with_capacity(nodes.len());

        for idx in 0..nodes.len() {
            let node = Node { id: idx };
            n_nodes[idx] = Arc::new(node);
            real_ids[idx] = nodes[idx];
            real_ids_to_local_ids.insert(nodes[idx], idx);
        }

        for idx in 0..edges.len() {
            let (left, right, weight) = edges[idx];
            let local_left = real_ids_to_local_ids.get(&left).unwrap();
            let local_right = real_ids_to_local_ids.get(&right).unwrap();
            let edge = Edge {
                left: n_nodes[*local_left].clone(),
                right: n_nodes[*local_right].clone(),
                weight,
            };
            n_edges[idx] = Arc::new(edge);

            connections[*local_left].push(n_edges[*local_right].clone());
            connections[*local_right].push(n_edges[*local_left].clone());
        }

        Ok(Graph {
            nodes: n_nodes,
            edges: n_edges,
            real_ids,
            connections,
        })
    }

    pub fn get_node(&self, id: usize) -> Arc<Node> {
        self.nodes[id].clone()
    }
    pub fn get_edge(&self, id: usize) -> Arc<Edge> {
        self.edges[id].clone()
    }
    pub fn get_availible_nodes_for_node(&self, node: Arc<Node>) -> Vec<Arc<Node>> {
        self.connections[node.id]
            .iter()
            .map(|e| {
                if node.id == e.left.id {
                    e.right.clone()
                } else {
                    e.left.clone()
                }
            })
            .collect()
    }

    pub fn distance(&self, node1: &Arc<Node>, node2: &Arc<Node>) -> Option<f32> {
        for conn in &self.connections[node1.id] {
            if conn.left.id == node2.id || conn.right.id == node2.id {
                return Some(conn.weight);
            }
        }
        return None;
    }
    pub fn delta(
        &self,
        pair1_left: &Arc<Node>,
        pair1_right: &Arc<Node>,
        pair2_left: &Arc<Node>,
        pair2_right: &Arc<Node>,
    ) -> Option<f32> {
        let old_pair1 = self.distance(pair1_left, pair1_right);
        let old_pair2 = self.distance(pair2_left, pair2_right);

        let new_pair1 = self.distance(pair1_left, pair2_left);
        let new_pair2 = self.distance(pair1_right, pair2_right);

        match (old_pair1, old_pair2, new_pair1, new_pair2) {
            (Some(old1), Some(old2), Some(new1), Some(new2)) => Some(new1 + new2 - old1 - old2),
            _ => None,
        }
    }

    fn verify_node_edges_correctness(nodes: &[usize], edges: &[(usize, usize, f32)]) -> bool {
        //TODO: complete?
        //TODO no duplicats?
        for (left, right, weight) in edges {
            if !nodes.contains(left) || !nodes.contains(right) {
                return false;
            }
        }
        true
    }
}
