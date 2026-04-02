use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct GraphCreationError;

impl Display for GraphCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "graph creation error")
    }
}
