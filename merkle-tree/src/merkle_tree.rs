use {std::fmt::Debug, solana_program::hash::{hashv, Hash}};

// We need to discern between leaf and intermediate nodes to prevent trivial second
// pre-image attacks.
// https://flawed.net.nz/2018/02/21/attacking-merkle-trees-with-a-second-preimage-attack
const LEAF_PREFIX: &[u8] = &[0];
const INTERMEDIATE_PREFIX: &[u8] = &[1];

macro_rules! hash_leaf {
    {$d:ident} => {
        hashv(&[LEAF_PREFIX, $d])
    }
}

macro_rules! hash_intermediate {
    {$l:ident, $r:ident} => {
        hashv(&[INTERMEDIATE_PREFIX, $l.as_ref(), $r.as_ref()])
    }
}

#[derive(Debug)]
pub struct MerkleTree {
    leaf_count: usize,
    nodes: Vec<Hash>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProofEntry<'a>(&'a Hash, Option<&'a Hash>, Option<&'a Hash>);

impl<'a> ProofEntry<'a> {
    pub fn new(
        target: &'a Hash,
        left_sibling: Option<&'a Hash>,
        right_sibling: Option<&'a Hash>,
    ) -> Self {
        assert!(left_sibling.is_none() ^ right_sibling.is_none());
        Self(target, left_sibling, right_sibling)
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Proof<'a>(Vec<ProofEntry<'a>>);

impl<'a> Proof<'a> {
    pub fn push(&mut self, entry: ProofEntry<'a>) {
        self.0.push(entry)
    }

    pub fn verify(&self, candidate: Hash) -> bool {
        let result = self.0.iter().try_fold(candidate, |candidate, pe| {
            let lsib = pe.1.unwrap_or(&candidate);
            let rsib = pe.2.unwrap_or(&candidate);
            let hash = hash_intermediate!(lsib, rsib);

            if hash == *pe.0 {
                Some(hash)
            } else {
                None
            }
        });
        matches!(result, Some(_))
    }
}

impl MerkleTree {
    #[inline]
    fn next_level_len(level_len: usize) -> usize {
        if level_len == 1 {
            0
        } else {
            (level_len + 1) / 2
        }
    }

    fn calculate_vec_capacity(leaf_count: usize) -> usize {
        // the most nodes consuming case is when n-1 is full balanced binary tree
        // then n will cause the previous tree add a left only path to the root
        // this cause the total nodes number increased by tree height, we use this
        // condition as the max nodes consuming case.
        // n is current leaf nodes number
        // assuming n-1 is a full balanced binary tree, n-1 tree nodes number will be
        // 2(n-1) - 1, n tree height is closed to log2(n) + 1
        // so the max nodes number is 2(n-1) - 1 + log2(n) + 1, finally we can use
        // 2n + log2(n+1) as a safe capacity value.
        // test results:
        // 8192 leaf nodes(full balanced):
        // computed cap is 16398, actually using is 16383
        // 8193 leaf nodes:(full balanced plus 1 leaf):
        // computed cap is 16400, actually using is 16398
        // about performance: current used fast_math log2 code is constant algo time
        if leaf_count > 0 {
            fast_math::log2_raw(leaf_count as f32) as usize + 2 * leaf_count + 1
        } else {
            0
        }
    }

    pub fn new<T: AsRef<[u8]>>(items: &[T]) -> Self {
        let cap = MerkleTree::calculate_vec_capacity(items.len());
        let mut mt = MerkleTree {
            leaf_count: items.len(),
            nodes: Vec::with_capacity(cap),
        };

        for item in items {
            let item = item.as_ref();
            let hash = hash_leaf!(item);
            mt.nodes.push(hash);
        }

        let mut level_len = MerkleTree::next_level_len(items.len());
        let mut level_start = items.len();
        let mut prev_level_len = items.len();
        let mut prev_level_start = 0;
        while level_len > 0 {
            for i in 0..level_len {
                let prev_level_idx = 2 * i;
                let lsib = &mt.nodes[prev_level_start + prev_level_idx];
                let rsib = if prev_level_idx + 1 < prev_level_len {
                    &mt.nodes[prev_level_start + prev_level_idx + 1]
                } else {
                    // Duplicate last entry if the level length is odd
                    &mt.nodes[prev_level_start + prev_level_idx]
                };

                let hash = hash_intermediate!(lsib, rsib);
                mt.nodes.push(hash);
            }
            prev_level_start = level_start;
            prev_level_len = level_len;
            level_start += level_len;
            level_len = MerkleTree::next_level_len(level_len);
        }

        mt
    }

    pub fn get_root(&self) -> Option<&Hash> {
        self.nodes.iter().last()
    }

    fn append_nodes(nodes: &mut [Hash], mut leaf_count: usize, leaves: Vec<Hash>) -> usize {
        for mut leaf in leaves {
            let mut layer: usize = 0;
            leaf_count += 1;
            let mut cursor = leaf_count;
            while (cursor & 1) == 0 {
                let arg1 = &leaf;
                let arg2 = &nodes[layer];
                //todo: verify and implement jump's fancy avx implementation
                leaf = hash_intermediate!(arg1, arg2);
                layer += 1;
                cursor >>= 1;
            }

            nodes[layer] = leaf;
        }
        leaf_count
    }

    fn private_depth(leaf_count: usize) -> usize {
        if leaf_count <= 1 {
            leaf_count
        } else {
            (63 - (leaf_count - 1).leading_zeros()) as usize + 2
        }
    }

    fn commit_finish(nodes: &mut [Hash], leaf_count: usize) -> Hash {
        let leaf_count = leaf_count;
        let root_idx: usize = Self::private_depth(leaf_count) - 1;

        if !leaf_count.is_power_of_two() {
            let mut layer = leaf_count.trailing_zeros() as usize;
            let mut layer_count = leaf_count >> layer;
            let mut tmp = nodes[layer];

            while layer_count > 1 {
                tmp = if (layer_count & 1) != 0 {
                    let arg1 = &tmp;
                    let arg2 = &tmp;
                    hash_intermediate!(arg1, arg2)
                } else {
                    let arg1 = &tmp;
                    let arg2 = &nodes[layer];
                    hash_intermediate!(arg1, arg2)
                };
                layer += 1;
                layer_count = (layer_count + 1) >> 1;
            }
            nodes[root_idx] = tmp;
        }
        nodes[root_idx]
    }

    #[allow(clippy::uninit_vec)]
    pub fn merkle_root<T: AsRef<[u8]> + Debug>(items: &[T]) -> Option<Hash> {
        if items.is_empty() {
            return None;
        }
        let mut nodes: Vec<Hash> = Vec::with_capacity(63);
        unsafe {
            nodes.set_len(63);
        }

        let hashes = items
            .iter()
            .map(|item| {
                let item = item.as_ref();
                hash_leaf!(item)
            })
            .collect::<Vec<_>>();
        let mut leaf_count = 0;
        leaf_count = Self::append_nodes(&mut nodes, leaf_count, hashes);

        let res = Self::commit_finish(&mut nodes, leaf_count);

        /*let temp: Hash = "11111111111111111111111111111111".parse().unwrap();

        if res == Hash::default() || res == temp {
            panic!("Bad hash, data: {:?}", items);
        }*/

        let tree = Self::new(items);
        let res2 = *tree.get_root().unwrap();
        if res != res2 {
            panic!("Bad hash, data: {:?}", items);
        }
        Some(res)
    }

    pub fn find_path(&self, index: usize) -> Option<Proof> {
        if index >= self.leaf_count {
            return None;
        }

        let mut level_len = self.leaf_count;
        let mut level_start = 0;
        let mut path = Proof::default();
        let mut node_index = index;
        let mut lsib = None;
        let mut rsib = None;
        while level_len > 0 {
            let level = &self.nodes[level_start..(level_start + level_len)];

            let target = &level[node_index];
            if lsib.is_some() || rsib.is_some() {
                path.push(ProofEntry::new(target, lsib, rsib));
            }
            if node_index % 2 == 0 {
                lsib = None;
                rsib = if node_index + 1 < level.len() {
                    Some(&level[node_index + 1])
                } else {
                    Some(&level[node_index])
                };
            } else {
                lsib = Some(&level[node_index - 1]);
                rsib = None;
            }
            node_index /= 2;

            level_start += level_len;
            level_len = MerkleTree::next_level_len(level_len);
        }
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST: &[&[u8]] = &[
        b"my", b"very", b"eager", b"mother", b"just", b"served", b"us", b"nine", b"pizzas",
        b"make", b"prime",
    ];
    const BAD: &[&[u8]] = &[b"bad", b"missing", b"false"];

    #[test]
    fn test_tree_from_empty() {
        let mt = MerkleTree::new::<[u8; 0]>(&[]);
        assert_eq!(mt.get_root(), None);
    }

    #[test]
    fn test_tree_from_one() {
        let input = b"test";
        let mt = MerkleTree::new(&[input]);
        let expected = hash_leaf!(input);
        assert_eq!(mt.get_root(), Some(&expected));
    }

    #[test]
    fn test_tree_from_many() {
        let mt = MerkleTree::new(TEST);
        // This golden hash will need to be updated whenever the contents of `TEST` change in any
        // way, including addition, removal and reordering or any of the tree calculation algo
        // changes
        let bytes = hex::decode("b40c847546fdceea166f927fc46c5ca33c3638236a36275c1346d3dffb84e1bc")
            .unwrap();
        let expected = Hash::new(&bytes);
        assert_eq!(mt.get_root(), Some(&expected));
    }

    #[test]
    fn test_path_creation() {
        let mt = MerkleTree::new(TEST);
        for (i, _s) in TEST.iter().enumerate() {
            let _path = mt.find_path(i).unwrap();
        }
    }

    #[test]
    fn test_path_creation_bad_index() {
        let mt = MerkleTree::new(TEST);
        assert_eq!(mt.find_path(TEST.len()), None);
    }

    #[test]
    fn test_path_verify_good() {
        let mt = MerkleTree::new(TEST);
        for (i, s) in TEST.iter().enumerate() {
            let hash = hash_leaf!(s);
            let path = mt.find_path(i).unwrap();
            assert!(path.verify(hash));
        }
    }

    #[test]
    fn test_path_verify_bad() {
        let mt = MerkleTree::new(TEST);
        for (i, s) in BAD.iter().enumerate() {
            let hash = hash_leaf!(s);
            let path = mt.find_path(i).unwrap();
            assert!(!path.verify(hash));
        }
    }

    #[test]
    fn test_proof_entry_instantiation_lsib_set() {
        ProofEntry::new(&Hash::default(), Some(&Hash::default()), None);
    }

    #[test]
    fn test_proof_entry_instantiation_rsib_set() {
        ProofEntry::new(&Hash::default(), None, Some(&Hash::default()));
    }

    #[test]
    fn test_nodes_capacity_compute() {
        let iteration_count = |mut leaf_count: usize| -> usize {
            let mut capacity = 0;
            while leaf_count > 0 {
                capacity += leaf_count;
                leaf_count = MerkleTree::next_level_len(leaf_count);
            }
            capacity
        };

        // test max 64k leaf nodes compute
        for leaf_count in 0..65536 {
            let math_count = MerkleTree::calculate_vec_capacity(leaf_count);
            let iter_count = iteration_count(leaf_count);
            assert!(math_count >= iter_count);
        }
    }

    #[test]
    #[should_panic]
    fn test_proof_entry_instantiation_both_clear() {
        ProofEntry::new(&Hash::default(), None, None);
    }

    #[test]
    #[should_panic]
    fn test_proof_entry_instantiation_both_set() {
        ProofEntry::new(
            &Hash::default(),
            Some(&Hash::default()),
            Some(&Hash::default()),
        );
    }
}
