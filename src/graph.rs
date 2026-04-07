use crate::errors::GraphCreationError;
use rand::RngExt;
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
    pub fn get_availible_nodes_for_node(&self, node: &Arc<Node>) -> Vec<Arc<Node>> {
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
    pub fn get_availible_nodes_with_dist_for_node(
        &self,
        node: &Arc<Node>,
    ) -> Vec<(Arc<Node>, f32)> {
        self.connections[node.id]
            .iter()
            .map(|e| {
                if node.id == e.left.id {
                    (e.right.clone(), e.weight)
                } else {
                    (e.left.clone(), e.weight)
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
        None
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

    pub fn initial_path(&self) -> (Vec<Arc<Node>>, Vec<Arc<Node>>, Vec<Arc<Node>>) {
        let mut visited = Vec::<Arc<Node>>::new();
        let mut not_visited = self.nodes.clone();
        let mut rng = rand::rng();
        let rand_idx = rng.random_range(0..self.nodes.len());
        let first = self.nodes[rand_idx].clone();

        visited.push(first.clone());

        let not_vis_idx = not_visited.iter().position(|e| e.id == first.id).unwrap(); // not_visited
                                                                                      // is the
                                                                                      // same as
                                                                                      // nodes,
                                                                                      // impossible
                                                                                      // not to
                                                                                      // find an
                                                                                      // element

        not_visited.remove(not_vis_idx);
        let mut path = vec![first];
        while !not_visited.is_empty() {
            let mut best_next: Option<Arc<Node>> = None;
            let mut best_dist: f32 = 0.0;
            for (right, dist) in self.get_availible_nodes_with_dist_for_node(path.last().unwrap()) {
                if visited.iter().find(|e| e.id == right.id).is_some() {
                    // if right is in visited,
                    // then we can't add it to
                    // path – skip
                    continue;
                }
                if best_next.is_none() {
                    best_next = Some(right);
                    best_dist = dist;
                    continue;
                }
                if Graph::left_better_than_right_f32(dist, best_dist) {
                    best_dist = dist;
                    best_next = Some(right);
                }
            }
            if best_next.is_none() {
                break;
            }

            let next = best_next.unwrap();
            path.push(next.clone());
            not_visited.remove(not_visited.iter().position(|e| e.id == next.id).unwrap());
            visited.push(next)
        }
        (path, visited, not_visited)
    }

    ///Calculates distance from a node to given path, returns None if there is no connections at
    ///all from the node to the path.
    pub fn distance_to_path(&self, node: &Arc<Node>, path: &Vec<Arc<Node>>) -> Option<f32> {
        let mut best_dist: Option<f32> = None;
        for p_node in path {
            let dist = self.distance(node, p_node);
            if dist.is_none() {
                continue;
            }
            if best_dist.is_none() {
                best_dist = dist;
                continue;
            }
            if Graph::left_better_than_right_f32(dist.unwrap(), best_dist.unwrap()) {
                best_dist = dist
            }
        }
        best_dist
    }

    /// Helping function that defines if it is maximization or minimization problem
    fn left_better_than_right_f32(left: f32, right: f32) -> bool {
        left > right // left > right then it is maximization problem
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
