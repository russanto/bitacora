use std::{borrow::BorrowMut, cell::RefCell};

use ethers::utils::keccak256;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::bytes::Bytes32;

pub trait Hasher {

    type ReturnType: AsRef<[u8]> + Clone + Eq + PartialOrd + Serialize + DeserializeOwned;

    fn hash<T: AsRef<[u8]>>(data: T) -> Self::ReturnType;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keccak256 {}

impl Hasher for Keccak256 {

    type ReturnType = Bytes32;

    fn hash<T: AsRef<[u8]>>(data: T) -> Self::ReturnType {
        keccak256(data).into()
    }
}

pub trait MerkleTree {
    type Node: AsRef<[u8]>;
    type Proof;

    fn root(&mut self) -> Option<Self::Node>;
    fn proof<E: AsRef<[u8]>>(&mut self, leaf: &E) -> Option<Self::Proof>;
    fn verify<E: AsRef<[u8]>>(&mut self, leaf: &E, proof: &[Self::Node]) -> bool;
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct MerkleTreeAppendOnly<H>
where
    H: Hasher
{
    nodes: Vec<H::ReturnType>,
    leaves: Vec<H::ReturnType>
}

impl <H: Hasher> Default for MerkleTreeAppendOnly<H> {
    fn default() -> Self {
        MerkleTreeAppendOnly {
            nodes: Vec::new(),
            leaves: Vec::new()
        }
    }
}

impl <H: Hasher> MerkleTreeAppendOnly<H> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append<T: AsRef<[u8]>>(&mut self, element: &T) -> usize {
        self.leaves.push(H::hash(element));
        self.leaves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.leaves.is_empty()
    }

    fn is_root_valid(&self) -> bool {
        self.leaves.len() == 0 && self.nodes.len() > 0
    }

    fn compute(&mut self) {
        // Reset the current tree
        let total_leaves = (self.nodes.len()+1)/2;
        self.nodes.truncate(total_leaves);
        // Include leaves appended after last computation
        self.nodes.append(&mut self.leaves);
        self.leaves.clear();

        let n_leaves = self.nodes.len();
        // With no leaves or one there is not anything to compute
        if n_leaves < 2 {
            return;
        }

        let mut nodes_start_index = 0;
        let mut odd_item_index: Option<usize> = None;
        loop {
            let nodes_end_index = self.nodes.len();
            for i in 0..(nodes_end_index-nodes_start_index)/2 {
                let base_index = nodes_start_index+(i*2);
                self.nodes.push(
                    Self::pairwise_hash(&self.nodes[base_index], &self.nodes[base_index+1])
                );
            }
            if odd_item_index.is_none() && nodes_end_index % 2 == 1 {
                odd_item_index = Some(nodes_end_index-1);
            }
            if nodes_end_index - nodes_start_index <= 2 {
                if odd_item_index.is_some() {
                    self.nodes.push(
                        Self::pairwise_hash(
                            self.nodes.last().unwrap(),
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

    pub fn verify_from_root<T: AsRef<[u8]>>(root: &H::ReturnType, leaf: &T, proof: &[H::ReturnType]) -> bool {
        let leaf_hash = H::hash(leaf);
        if proof.is_empty() {
            return *root == leaf_hash;
        }
        let mut accumulator = leaf_hash.clone();
        for proof_component in proof {
            accumulator = Self::pairwise_hash(&accumulator, proof_component);
        }
        accumulator == *root
    }
}

impl <H: Hasher> MerkleTree for MerkleTreeAppendOnly<H> {

    type Node = H::ReturnType;
    type Proof = Vec<H::ReturnType>;

    fn root(&mut self) -> Option<Self::Node> {
        if !self.is_root_valid() {
            self.compute();
        }
        self.nodes.last().cloned()
    }

    fn proof<T: AsRef<[u8]>>(&mut self, leaf: &T) -> Option<Self::Proof> {
        if self.is_empty() {
            return None;
        }
        if !self.is_root_valid() {
            self.compute();
        }
        let leaf_hash = H::hash(leaf);
        let mut item_cursor = match self.nodes.iter().position(|x| x == &leaf_hash) {
            Some(item_index) => item_index,
            None => return None
        };
        let mut proof = Vec::new();
        let n_leaves = (self.nodes.len()+1)/2;
        let tree_depth = if n_leaves.is_power_of_two() {
            n_leaves.ilog2()
        } else {
            n_leaves.ilog2()+1
        };
        let mut start_index = 0;
        let mut end_index = n_leaves;
        let mut odd_levels_count = 0;
        let mut odd_element_index = 0;
        let mut odd_element_rebalance = 0;
        let mut item_index = start_index+item_cursor;
        for _ in 0..tree_depth {
            if (end_index - start_index) % 2 == 1 {
                odd_levels_count += 1;
                if odd_levels_count % 2 == 0 {
                    odd_element_rebalance = 1;
                } else {
                    odd_element_index = end_index-1;
                }
            } 
            if item_cursor % 2 == 1 {
                proof.push(self.nodes[item_index-1].clone());
            } else {
                if item_index < end_index-1 {
                    proof.push(self.nodes[item_index+1].clone())
                } else {
                    if odd_levels_count % 2 == 0 {
                        proof.push(self.nodes[odd_element_index].clone());
                    }
                }
            }
            let n_next_level = (end_index-start_index)/2;
            start_index = end_index;
            end_index += n_next_level + odd_element_rebalance;
            odd_element_rebalance = 0;
            item_cursor /= 2;
            item_index = start_index+item_cursor;
        }
        Some(proof)
    }

    fn verify<T: AsRef<[u8]>>(&mut self, leaf: &T, proof: &[Self::Node]) -> bool {
        if self.is_empty() {
            return false;
        }
        if !self.is_root_valid() {
            self.compute();
        }
        match self.root() {
            Some(root) => Self::verify_from_root(&root, leaf, proof),
            None => false
        }
    }
}

// A stub with interior mutability in order to have a MerkleTree trait without mut in getters
// #[derive(Clone, Debug, Serialize)]
// pub struct MerkleTreeAppendOnly<H>
// where
//     H: Hasher
// {
//     nodes: RefCell<Vec<H::ReturnType>>,
//     leaves: RefCell<Vec<H::ReturnType>>
// }

// impl <H: Hasher> Default for MerkleTreeAppendOnly<H> {
//     fn default() -> Self {
//         MerkleTreeAppendOnly {
//             nodes: RefCell::new(Vec::new()),
//             leaves: RefCell::new(Vec::new())
//         }
//     }
// }

// impl <H: Hasher> MerkleTreeAppendOnly<H> {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     pub fn append<T: AsRef<[u8]>>(&mut self, element: T) -> usize {
//         let mut leaves = self.leaves.borrow_mut();
//         leaves.push(H::hash(element));
//         leaves.len()
//     }

//     pub fn is_empty(&self) -> bool {
//         self.nodes.borrow().is_empty() && self.leaves.borrow().is_empty()
//     }

//     fn is_root_valid(&self) -> bool {
//         self.leaves.borrow().len() == 0 && self.nodes.borrow().len() > 0
//     }

//     fn compute(&self) {
//         let mut nodes = self.nodes.borrow_mut();
//         let mut leaves = self.leaves.borrow_mut();

//         // Reset the current tree
//         let total_leaves = (nodes.len()+1)/2;
//         nodes.truncate(total_leaves);
//         // Include leaves appended after last computation
//         nodes.append(&mut leaves);
//         leaves.clear();

//         let n_leaves = nodes.len();
//         // With no leaves or one there is not anything to compute
//         if n_leaves < 2 {
//             return;
//         }

//         let mut nodes_start_index = 0;
//         let mut odd_item_index: Option<usize> = None;
//         loop {
//             let nodes_end_index = nodes.len();
//             for i in 0..(nodes_end_index-nodes_start_index)/2 {
//                 let base_index = nodes_start_index+(i*2);
//                 nodes.push(
//                     Self::pairwise_hash(&nodes[base_index], &nodes[base_index+1])
//                 );
//             }
//             if odd_item_index.is_none() && nodes_end_index % 2 == 1 {
//                 odd_item_index = Some(nodes_end_index-1);
//             }
//             if nodes_end_index - nodes_start_index <= 2 {
//                 if odd_item_index.is_some() {
//                     nodes.push(
//                         Self::pairwise_hash(
//                             nodes.last().unwrap(),
//                             &nodes[odd_item_index.unwrap()]
//                         )
//                     );
//                 }
//                 return;
//             }
//             nodes_start_index = nodes_end_index;
//         }
//     }

//     fn pairwise_hash(v1: &H::ReturnType, v2: &H::ReturnType) -> H::ReturnType {
//         let mut hash_buffer = Vec::new(); // TODO: make it reusable and remove from heap
//         if v1 < v2 {
//             hash_buffer.extend_from_slice(v1.as_ref());
//             hash_buffer.extend_from_slice(v2.as_ref());
//         } else {
//             hash_buffer.extend_from_slice(v2.as_ref());
//             hash_buffer.extend_from_slice(v1.as_ref());
//         }
//         H::hash(hash_buffer)
//     }
// }

// impl <H: Hasher, T: AsRef<[u8]>> MerkleTree<T> for MerkleTreeAppendOnly<H> {

//     type Node = H::ReturnType;

//     fn root(&self) -> Option<Self::Node> {
//         if !self.is_root_valid() {
//             self.compute();
//         }
//         self.nodes.borrow().last().cloned()
//     }

//     fn proof(&self, leaf: &T) -> Option<Vec<Self::Node>> {
//         let nodes = self.nodes.borrow();
//         if self.is_empty() {
//             return None;
//         }
//         if !self.is_root_valid() {
//             self.compute();
//         }
//         let leaf_hash = H::hash(leaf);
//         let mut item_cursor = match nodes.iter().position(|x| x == &leaf_hash) {
//             Some(item_index) => item_index,
//             None => return None
//         };
//         let mut proof = Vec::new();
//         let n_leaves = (nodes.len()+1)/2;
//         let tree_depth = if n_leaves.is_power_of_two() {
//             n_leaves.ilog2()
//         } else {
//             n_leaves.ilog2()+1
//         };
//         let mut start_index = 0;
//         let mut end_index = n_leaves;
//         let mut odd_levels_count = 0;
//         let mut odd_element_index = 0;
//         let mut odd_element_rebalance = 0;
//         let mut item_index = start_index+item_cursor;
//         for _ in 0..tree_depth {
//             if (end_index - start_index) % 2 == 1 {
//                 odd_levels_count += 1;
//                 if odd_levels_count % 2 == 0 {
//                     odd_element_rebalance = 1;
//                 } else {
//                     odd_element_index = end_index-1;
//                 }
//             } 
//             if item_cursor % 2 == 1 {
//                 proof.push(nodes[item_index-1].clone());
//             } else {
//                 if item_index < end_index-1 {
//                     proof.push(nodes[item_index+1].clone())
//                 } else {
//                     if odd_levels_count % 2 == 0 {
//                         proof.push(nodes[odd_element_index].clone());
//                     }
//                 }
//             }
//             let n_next_level = (end_index-start_index)/2;
//             start_index = end_index;
//             end_index += n_next_level + odd_element_rebalance;
//             odd_element_rebalance = 0;
//             item_cursor /= 2;
//             item_index = start_index+item_cursor;
//         }
//         Some(proof)
//     }

//     fn verify(&self, leaf: &T, proof: &[Self::Node]) -> bool {
//         let nodes = self.nodes.borrow();
//         if self.is_empty() {
//             return false;
//         }
//         if !self.is_root_valid() {
//             self.compute();
//         }
//         let leaf_hash = H::hash(leaf);
//         if proof.is_empty() {
//             return nodes.len() == 1 && nodes[0] == leaf_hash;   
//         }
//         let mut accumulator = leaf_hash.clone();
//         for proof_component in proof.iter() {
//             accumulator = Self::pairwise_hash(&accumulator, proof_component);
//         }
//         match (self as &dyn MerkleTree<T, Node = Self::Node>).root() {
//             Some(root) => accumulator == root,
//             None => false
//         }
//     }
// }

pub type MerkleTreeOZ = MerkleTreeAppendOnly<Keccak256>;

#[cfg(test)]
mod test {
    use crate::common::prelude::{Hasher, Bytes32};

    use super::{MerkleTree, MerkleTreeAppendOnly, Keccak256};

    impl MerkleTreeAppendOnly<Keccak256> {
        fn test_instance() -> Self {
            let values = vec!["a", "b", "c", "d", "e", "f"];
            let mut mt = MerkleTreeAppendOnly::<Keccak256>::new();
            values.iter().for_each(|v| {
                mt.append(v);
            });
            mt
        }
    }

    #[test]
    fn test_merkle_tree_with_odd_elements() {
        let values = vec!["a", "b", "c", "d", "e"];
        let expected_root = "1dd0d2a6ae466d665cb26e1a31f07c57ae5df7d2bc559cd5826d417be9141a5d";
        let leaves_to_prove = vec!["b", "e"];
        let expected_proofs = vec![
            vec![
                Bytes32::try_from("3ac225168df54212a25c1c01fd35bebfea408fdac2e31ddd6f80a4bbf9a5f1cb").unwrap(),
                Bytes32::try_from("d253a52d4cb00de2895e85f2529e2976e6aaaa5c18106b68ab66813e14415669").unwrap(),
                Bytes32::try_from("a8982c89d80987fb9a510e25981ee9170206be21af3c8e0eb312ef1d3382e761").unwrap()
            ],
            vec![
                Bytes32::try_from("68203f90e9d07dc5859259d7536e87a6ba9d345f2552b5b9de2999ddce9ce1bf").unwrap()
            ]
        ];
        let mut mt = MerkleTreeAppendOnly::<Keccak256>::new();
        values.iter().for_each(|v| {
            mt.append(v);
        });
        assert!(!mt.is_root_valid(), "Root is flagged as valid after tree modification");
        assert_eq!(
            Bytes32::try_from(expected_root).unwrap(),
            mt.root().unwrap(),
            "Merkle root mismatch"
        );
        assert!(mt.is_root_valid(), "Root is flagged as invalid after root calculation");
        assert_eq!(mt.nodes.len(), values.len()*2-1, "Merkle Tree has more nodes than expected");
        for i in 0..leaves_to_prove.len() {
            let proof = mt.proof(&leaves_to_prove[i]).unwrap();
            for j in 0..std::cmp::max(expected_proofs[i].len(), proof.len()) {
                assert_eq!(
                    proof[j],
                    expected_proofs[i][j],
                    "Proofs for test leaf {} differ at index {}", i, j 
                );
            }
            assert!(mt.verify(&leaves_to_prove[i], &proof), "Proof verification failed");
        }
    }

    #[test]
    fn test_merkle_tree_with_even_elements() {
        let values = vec!["a", "b", "c", "d", "e", "f"];
        let expected_root = "9012f1e18a87790d2e01faace75aaaca38e53df437cdce2c0552464dda4af49c";
        let leaves_to_prove = vec!["c", "f"];
        let expected_proofs = vec![
            vec![
                Bytes32::try_from("f1918e8562236eb17adc8502332f4c9c82bc14e19bfc0aa10ab674ff75b3d2f3").unwrap(),
                Bytes32::try_from("805b21d846b189efaeb0377d6bb0d201b3872a363e607c25088f025b0c6ae1f8").unwrap(),
                Bytes32::try_from("f0b49bb4b0d9396e0315755ceafaa280707b32e75e6c9053f5cdf2679dcd5c6a").unwrap()
            ],
            vec![
                Bytes32::try_from("a8982c89d80987fb9a510e25981ee9170206be21af3c8e0eb312ef1d3382e761").unwrap(),
                Bytes32::try_from("68203f90e9d07dc5859259d7536e87a6ba9d345f2552b5b9de2999ddce9ce1bf").unwrap()
            ]
        ];
        let mut mt = MerkleTreeAppendOnly::<Keccak256>::new();
        values.iter().for_each(|v| {
            mt.append(v);
        });
        assert!(!mt.is_root_valid(), "Root is flagged as valid after tree modification");
        assert_eq!(
            Bytes32::try_from(expected_root).unwrap(),
            mt.root().unwrap(),
            "Merkle root mismatch"
        );
        assert!(mt.is_root_valid(), "Root is flagged as invalid after root calculation");
        assert_eq!(mt.nodes.len(), values.len()*2-1, "Merkle Tree has more nodes than expected");
        for i in 0..leaves_to_prove.len() {
            let proof = mt.proof(&leaves_to_prove[i]).unwrap();
            for j in 0..std::cmp::max(expected_proofs[i].len(), proof.len()) {
                assert_eq!(
                    proof[j],
                    expected_proofs[i][j],
                    "Proofs for test leaf {} differ at index {}", i, j 
                );
            }
            assert!(mt.verify(&leaves_to_prove[i], &proof), "Proof verification failed");
        }
    }

    #[test]
    fn test_merkle_tree_with_one_element() {
        let values = vec!["a"];
        let expected_root = Bytes32::try_from("3ac225168df54212a25c1c01fd35bebfea408fdac2e31ddd6f80a4bbf9a5f1cb").unwrap();
        let mut mt = MerkleTreeAppendOnly::<Keccak256>::new();
        values.iter().for_each(|v| {
            mt.append(v);
        });
        assert!(!mt.is_root_valid(), "Root is flagged as valid after tree modification");
        assert_eq!(
            expected_root,
            mt.root().unwrap(),
            "Computed root is not the expected one"
        );
        assert!(mt.is_root_valid(), "Root is flagged as invalid with no tree modification");
        assert_eq!(mt.nodes.len(), values.len()*2-1, "Merkle Tree has more nodes than expected");
        assert!(mt.verify(&values[0], &Vec::<Bytes32>::new()));
    }

    #[test]
    fn test_merkle_root_with_no_elements() {
        let mut mt = MerkleTreeAppendOnly::<Keccak256>::new();
        assert!(!mt.is_root_valid(), "Root is flagged as valid for empty tree");
        assert!(mt.root().is_none());
        assert!(!mt.is_root_valid(), "Root is flagged as valid for empty tree after root request");
    }
    
}