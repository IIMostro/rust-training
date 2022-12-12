use crate::{Block, BlockHash};
use rayon::prelude::*;

pub const PREFIX_ZERO: &[u8] = &[0, 0, 0];

pub fn pow(block: Block) -> Option<BlockHash>{
    let base_hasher = blake3_base_hash(&block.data);
    // 从0循环到 u32最大的4294967295， 每次循环都会计算一次hash, 如果hash的值前两位是0，就返回hash
    let nonce = (0..u32::MAX).find(|n| {
        let hash = blake3_hash(base_hasher.clone(), *n);
        &hash[.. PREFIX_ZERO.len()]== PREFIX_ZERO
    });
    nonce.map(|x| {
        let id = blake3::hash(&block.data).as_bytes().to_vec();
        let value =  blake3_hash(base_hasher, x);
        BlockHash{
            id,
            hash: value,
            nonce: x
        }
    })
}

pub fn pow_v2(block: Block) -> Option<BlockHash>{
    let base_hasher = blake3_base_hash(&block.data);
    // 并行计算，使用rayon计算，不能使用find, 需要使用到find_any, find_any会并行计算，只要有一个满足条件就返回
    (0..u32::MAX).into_par_iter().find_any(|n| {
        let hash = blake3_hash(base_hasher.clone(), *n);
        &hash[.. PREFIX_ZERO.len()]== PREFIX_ZERO
    }).map(|x| {
        BlockHash{
            id: blake3::hash(&block.data).as_bytes().to_vec(),
            hash: blake3_hash(base_hasher, x),
            nonce: x
        }
    })
}

fn blake3_hash(mut hasher: blake3::Hasher, nonce: u32) -> Vec<u8>{
    hasher.update(&nonce.to_be_bytes()[..]);
    hasher.finalize().as_bytes().to_vec()
}

fn blake3_base_hash(data: &[u8]) -> blake3::Hasher{
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    hasher.clone()
}