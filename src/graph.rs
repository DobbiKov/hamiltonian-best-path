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
            n_nodes.push(Arc::new(node));
            real_ids.push(nodes[idx]);
            real_ids_to_local_ids.insert(nodes[idx], idx);
            connections.push(Vec::<Arc<Edge>>::new());
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
            let a_e = Arc::new(edge);
            n_edges.push(a_e.clone());
            connections[*local_left].push(a_e.clone());
            connections[*local_right].push(a_e);
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

    fn find_idx_that_improve_path(&self, path: &Vec<Arc<Node>>) -> Option<(usize, usize)> {
        let mut best_pair_idx: Option<(usize, usize)> = None;
        let mut best_delta: Option<f32> = None;

        let i_lim: usize = if path.len() >= 3 { path.len() - 3 } else { 0 };
        for i in 0..i_lim {
            let j_lim: usize = if path.len() >= 1 { path.len() - 1 } else { 0 };
            for j in (i + 2)..j_lim {
                let delta = self.delta(&path[i], &path[i + 1], &path[j], &path[j + 1]);
                if delta.is_none() {
                    continue;
                }
                if !Graph::left_better_than_right_f32(delta.unwrap(), 0.0) {
                    continue;
                }
                if best_pair_idx.is_none() {
                    best_pair_idx = Some((i, j));
                    best_delta = delta;
                    continue;
                }
                if best_delta.unwrap() != delta.unwrap()
                    && Graph::left_better_than_right_f32(delta.unwrap(), best_delta.unwrap())
                {
                    best_pair_idx = Some((i, j));
                    best_delta = delta;
                    continue;
                }
            }
        }
        best_pair_idx
    }

    fn improve_path(&self, path: Vec<Arc<Node>>) -> (Vec<Arc<Node>>, bool) {
        let best_pair_idx: Option<(usize, usize)> = self.find_idx_that_improve_path(&path);
        if best_pair_idx.is_none() {
            return (path, false);
        }

        let (i, j) = best_pair_idx.unwrap();
        let mut new_path = Vec::<Arc<Node>>::new();
        for idx in 0..i + 1 {
            new_path.push(path[idx].clone())
        }
        for idx in (i + 1..j + 1).rev() {
            new_path.push(path[idx].clone());
        }
        for idx in (j + 1..path.len()) {
            new_path.push(path[idx].clone())
        }

        (new_path, true)
    }

    fn improve_while_possible(&self, path: Vec<Arc<Node>>) -> Vec<Arc<Node>> {
        let (mut new_path, mut changed) = self.improve_path(path);
        while changed {
            (new_path, changed) = self.improve_path(new_path);
        }
        new_path
    }

    pub fn build_approx_best_path(&self) -> Vec<Arc<Node>> {
        let (mut path, _, _) = self.initial_path();
        path = self.improve_while_possible(path);
        path
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

#[cfg(test)]
mod tests {
    use super::Graph; // import everything from the parent module

    #[test]
    fn test_simple_graph() {
        let g = Graph::new(&[5, 8, 11], &[(5, 8, 1.5), (8, 11, 2.5)]).unwrap();

        // real_ids contains all provided node ids
        assert!(g.real_ids.contains(&5));
        assert!(g.real_ids.contains(&8));
        assert!(g.real_ids.contains(&11));

        assert_eq!(g.nodes.len(), 3);
        assert_eq!(g.edges.len(), 2);

        // resolve local indices from real ids
        let id_5 = g.real_ids.iter().position(|e| *e == 5).unwrap();
        let id_8 = g.real_ids.iter().position(|e| *e == 8).unwrap();
        let id_11 = g.real_ids.iter().position(|e| *e == 11).unwrap();

        // local ids are distinct
        assert_ne!(id_5, id_8);
        assert_ne!(id_8, id_11);
        assert_ne!(id_5, id_11);

        assert_eq!(g.connections.len(), 3);

        // connections: node 5 and 11 each touch 1 edge; node 8 touches both
        assert_eq!(g.connections[id_5].len(), 1);
        assert_eq!(g.connections[id_8].len(), 2);
        assert_eq!(g.connections[id_11].len(), 1);

        // get_node returns a node with the correct local id
        let node_5 = g.get_node(id_5);
        let node_8 = g.get_node(id_8);
        let node_11 = g.get_node(id_11);
        assert_eq!(node_5.id, id_5);
        assert_eq!(node_8.id, id_8);
        assert_eq!(node_11.id, id_11);

        // get_edge returns edges with the expected weights
        // edges are stored in insertion order: (5,8,1.5) then (8,11,2.5)
        assert_eq!(g.get_edge(0).weight, 1.5);
        assert_eq!(g.get_edge(1).weight, 2.5);

        // get_availible_nodes_for_node
        let neighbors_5 = g.get_availible_nodes_for_node(&node_5);
        assert_eq!(neighbors_5.len(), 1);
        assert_eq!(neighbors_5[0].id, id_8);

        let neighbors_11 = g.get_availible_nodes_for_node(&node_11);
        assert_eq!(neighbors_11.len(), 1);
        assert_eq!(neighbors_11[0].id, id_8);

        let neighbors_8 = g.get_availible_nodes_for_node(&node_8);
        assert_eq!(neighbors_8.len(), 2);
        let neighbor_ids: Vec<usize> = neighbors_8.iter().map(|n| n.id).collect();
        assert!(neighbor_ids.contains(&id_5));
        assert!(neighbor_ids.contains(&id_11));

        // get_availible_nodes_with_dist_for_node
        let neighbors_8_dist = g.get_availible_nodes_with_dist_for_node(&node_8);
        assert_eq!(neighbors_8_dist.len(), 2);
        let has_5_with_1_5 = neighbors_8_dist
            .iter()
            .any(|(n, d)| n.id == id_5 && *d == 1.5);
        let has_11_with_2_5 = neighbors_8_dist
            .iter()
            .any(|(n, d)| n.id == id_11 && *d == 2.5);
        assert!(has_5_with_1_5);
        assert!(has_11_with_2_5);

        // distance
        assert_eq!(g.distance(&node_5, &node_8), Some(1.5));
        assert_eq!(g.distance(&node_8, &node_5), Some(1.5)); // symmetric
        assert_eq!(g.distance(&node_8, &node_11), Some(2.5));
        assert_eq!(g.distance(&node_5, &node_11), None); // no direct edge

        // delta: swapping (5->8, 8->11) into (5->8, 8->11) — same pairs — yields 0.0
        let d = g.delta(&node_5, &node_8, &node_8, &node_11);
        assert_eq!(d, Some(0.0));

        // delta returns None when a required edge does not exist
        let d_none = g.delta(&node_5, &node_11, &node_8, &node_11);
        assert_eq!(d_none, None);

        // distance_to_path: best (max) reachable distance from node_5 to [node_8, node_11]
        // distance(5,8)=1.5, distance(5,11)=None → best = 1.5
        let path_8_11 = vec![node_8.clone(), node_11.clone()];
        assert_eq!(g.distance_to_path(&node_5, &path_8_11), Some(1.5));

        // distance_to_path: node_8 reaches node_5 (1.5) and node_11 (2.5) → best = 2.5
        let path_5_11 = vec![node_5.clone(), node_11.clone()];
        assert_eq!(g.distance_to_path(&node_8, &path_5_11), Some(2.5));

        // distance_to_path returns None when no connection exists to any path node
        let path_only_5 = vec![node_5.clone()];
        assert_eq!(g.distance_to_path(&node_11, &path_only_5), None);

        // build_approx_best_path visits all nodes exactly once
        let path = g.build_approx_best_path();
        assert_eq!(path.len(), 3);
        let path_ids: Vec<usize> = path.iter().map(|n| n.id).collect();
        assert!(path_ids.contains(&id_5));
        assert!(path_ids.contains(&id_8));
        assert!(path_ids.contains(&id_11));
    }

    #[test]
    fn test_invalid_edge_returns_error() {
        // edge references node id 99 which is not in the node list
        let result = Graph::new(&[5, 8], &[(5, 99, 1.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_approx_best_path_complete_graph() {
        // 4-node complete graph: every node is connected to every other,
        // so initial_path always visits all 4 nodes regardless of starting point.
        let g = Graph::new(
            &[0, 1, 2, 3],
            &[
                (0, 1, 1.0),
                (0, 2, 2.0),
                (0, 3, 3.0),
                (1, 2, 4.0),
                (1, 3, 5.0),
                (2, 3, 6.0),
            ],
        )
        .unwrap();

        let path = g.build_approx_best_path();

        // path visits all 4 nodes exactly once
        assert_eq!(path.len(), 4);
        let mut ids: Vec<usize> = path.iter().map(|n| n.id).collect();
        ids.sort();
        assert_eq!(ids, vec![0, 1, 2, 3]);
    }
}
