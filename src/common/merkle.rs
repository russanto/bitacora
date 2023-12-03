use ethers::utils::keccak256;

use super::bytes::Bytes32;

pub trait Hasher {

    type ReturnType: AsRef<[u8]> + PartialOrd;

    fn hash<T: AsRef<[u8]>>(data: T) -> Self::ReturnType;
}

pub struct Keccak256 {}

impl Hasher for Keccak256 {

    type ReturnType = Bytes32;

    fn hash<T: AsRef<[u8]>>(data: T) -> Self::ReturnType {
        keccak256(data).into()
    }
}

pub type MerkleRoot = Bytes32;

#[derive(Clone, Debug)]
pub struct MerkleTree<H>
where
    H: Hasher
{
    nodes: Vec<H::ReturnType>,
    leaves: Vec<H::ReturnType>
}

impl <H: Hasher> Default for MerkleTree<H> {
    fn default() -> Self {
        MerkleTree {
            nodes: Vec::new(),
            leaves: Vec::new()
        }
    }
}

impl <H: Hasher> MerkleTree<H> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append<T: AsRef<[u8]>>(&mut self, element: &T) -> usize {
        self.leaves.push(H::hash(element));
        self.leaves.len()
    }

    pub fn root(&mut self) -> Option<&H::ReturnType> {
        if !self.is_root_valid() {
            let total_leaves = (self.nodes.len()+1)/2;
            self.nodes.truncate(total_leaves);
            self.nodes.append(&mut self.leaves);
            self.leaves.clear();
            self.compute();
        }
        self.nodes.last()
    }

    fn is_root_valid(&self) -> bool {
        self.leaves.len() == 0 && self.nodes.len() > 0
    }

    fn compute(&mut self) {
        if self.nodes.len() == 1 {
            return;
        }
        let mut nodes_start_index = 0;
        let mut nodes_end_index = 0;
        let mut base_index = 0;
        let mut odd_item_index: Option<usize> = None;
        loop {
            nodes_end_index = self.nodes.len();
            for i in 0..(nodes_end_index-nodes_start_index)/2 {
                base_index = nodes_start_index+(i*2);
                self.nodes.push(
                    Self::pairwise_hash(&self.nodes[base_index], &self.nodes[base_index+1])
                );
            }
            if odd_item_index.is_none() && nodes_end_index % 2 == 1 {
                odd_item_index = Some(nodes_end_index-1);
            }
            if nodes_end_index - nodes_start_index == 2 {
                if odd_item_index.is_some() {
                    self.nodes.push(
                        Self::pairwise_hash(
                            &self.nodes[nodes_end_index],
                            &self.nodes[odd_item_index.unwrap()]
                        )
                    );
                }
                return;
            }
            nodes_start_index = nodes_end_index;
        }
    }

    fn pairwise_hash(v1: &H::ReturnType, v2: &H::ReturnType) -> H::ReturnType {
        let mut hash_buffer = Vec::new(); // TODO: make it reusable and remove from heap
        if v1 < v2 {
            hash_buffer.extend_from_slice(v1.as_ref());
            hash_buffer.extend_from_slice(v2.as_ref());
        } else {
            hash_buffer.extend_from_slice(v2.as_ref());
            hash_buffer.extend_from_slice(v1.as_ref());
        }
        H::hash(hash_buffer)
    }
}

#[cfg(test)]
mod test {
    use super::{MerkleTree, Keccak256};


    #[test]
    fn create_merkle_tree() {
        let values = vec!["a", "b", "c", "d", "e"];
        let mt = MerkleTree::<Keccak256>::new();
        values.iter().map(|v| mt.append(v));
        println!("{}", mt.root().unwrap());
    }
    
}