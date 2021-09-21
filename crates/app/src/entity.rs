use std::fmt::Debug;

use util::sparse_set::SparseIndex;

#[derive(Clone, Copy)]
pub struct Entity {
    id: u32,
    pub generate: u32,
}

impl Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ent{}", self.id)
    }
}

impl Entity {
    pub fn new(id: u32, generate: u32) -> Self {
        Entity { id, generate }
    }
}

impl SparseIndex for Entity {
    fn get_sparse_index(&self) -> usize {
        self.id as usize
    }
}
