use std::collections::HashMap;
use std::hash::Hash;

pub type CID = usize;
pub struct ConstantTracker<C>
where
    C: Hash,
{
    lookup: HashMap<C, CID>,
    constants: Vec<C>,
}

impl<C> ConstantTracker<C>
where
    C: Eq + Hash + Clone,
{
    pub fn new() -> ConstantTracker<C> {
        ConstantTracker { lookup: HashMap::new(), constants: Vec::new() }
    }

    pub fn get_id(&self, constant: &C) -> Option<CID> {
        self.lookup.get(constant).cloned()
    }

    pub fn insert(&mut self, constant: C) -> CID {
        if let Some(id) = self.get_id(&constant) {
            id
        } else {
            let id = self.constants.len();
            self.constants.push(constant.clone());
            self.lookup.insert(constant, id);
            id
        }
    }

    pub fn get_constant(&self, id: CID) -> Option<&C> {
        if id < self.constants.len() {
            Some(&self.constants[id])
        } else {
            None
        }
    }

    pub fn next_id(&self) -> CID {
        self.constants.len()
    }
}
