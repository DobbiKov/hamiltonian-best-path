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
        let mut real_ids_to_local_ids: HashMap<usize, usize> = HashMap::with_capacity(nodes.len())

        let mut n_edges: Vec<Arc<Edge>> = Vec::with_capacity(edges.len());
        let mut connections: Vec<Vec<Arc<Edge>>> = Vec::with_capacity(nodes.len());

        for idx in 0..nodes.len() {
            let node = Node{id: idx}
            n_nodes[idx] = Arc::new(node)
            real_ids[idx] = nodes[idx]
            real_ids_to_local_ids.insert(nodes[idx], idx);
        }

        for idx in 0..edges.len(){
            let (left, right, weight) = edges[idx];
            let local_left = real_ids_to_local_ids.get(&left).unwrap();
            let local_right = real_ids_to_local_ids.get(&right).unwrap();
            let edge = Edge{
                left: n_nodes[*local_left],
                right: n_nodes[*local_right],
                weight
            };
            n_edges[idx] = Arc::new(edge);

            connections[*local_left].push(n_edges[*local_right]);
            connections[*local_right].push(n_edges[*local_left]);
        }

        Ok(Graph{
            nodes: n_nodes,
            edges: n_edges,
            real_ids,
            connections
        })
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
