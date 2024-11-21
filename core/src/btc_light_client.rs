use crate::sha256_merkle::get_merkle_root;
use crate::constants::EXPECTED_EPOCH_SECONDS;

use crypto_bigint::U256;
use crypto_bigint::CheckedMul;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub trait AsLittleEndianBytes {
    fn to_little_endian(self) -> Self;
}

impl<const N: usize> AsLittleEndianBytes for [u8; N] {
    fn to_little_endian(mut self) -> Self {
        self.reverse();
        self
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Block {
    pub height: u64,
    pub version: [u8; 4],
    pub prev_blockhash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub time: [u8; 4],
    pub bits: [u8; 4],
    pub nonce: [u8; 4],
}

impl Block {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(80);
        bytes.extend_from_slice(&self.version);
        bytes.extend_from_slice(&self.prev_blockhash);
        bytes.extend_from_slice(&self.merkle_root);
        bytes.extend_from_slice(&self.time);
        bytes.extend_from_slice(&self.bits);
        bytes.extend_from_slice(&self.nonce);

        assert_eq!(bytes.len(), 80, "Header must be exactly 80 bytes");
        bytes
    }

    pub fn compute_block_hash(&self) -> [u8; 32] {
        let header = self.serialize();
        let first_hash = Sha256::digest(header);
        let second_hash: [u8; 32] = Sha256::digest(first_hash).into();
        second_hash.to_little_endian()
    }
}

// taken from rust-bitcoin
pub fn bits_to_target(bits: [u8; 4]) -> U256 {
    let bits = u32::from_le_bytes(bits);
    let (mant, expt) = {
        let unshifted_expt = bits >> 24;
        if unshifted_expt <= 3 {
            ((bits & 0xFFFFFF) >> (8 * (3 - unshifted_expt as usize)), 0)
        } else {
            (bits & 0xFFFFFF, 8 * ((bits >> 24) - 3))
        }
    };
    if mant > 0x7F_FFFF {
        U256::ZERO
    } else {
        U256::from(mant) << expt as usize
    }
}

pub fn assert_pow(proposed_block_hash: &[u8; 32], proposed_block: &Block, proposed_target: U256) {
    let calculated_block_hash = proposed_block.compute_block_hash();

    // [2] verify proposed block hash matches calculated block hash
    assert_eq!(calculated_block_hash, *proposed_block_hash);

    // [3] verify PoW -> block hash <= proposed target
    assert!(
        U256::from_be_slice(proposed_block_hash).le(&proposed_target),
        "PoW invalid hash < target"
    );
}

pub fn verify_block(
    proposed_block_hash: [u8; 32],
    previous_block_hash: [u8; 32],
    proposed_block: &Block,
    retarget_block: &Block,
    previous_block_height: u64,
) {
    // [1] verify proposed target is equal to real target
    let proposed_target = bits_to_target(proposed_block.bits);
    assert_eq!(
        retarget_block.bits, proposed_block.bits,
        "Proposed target does not match real target"
    );

    // [2] verify the proposed block height is one greater than previous_block_height
    assert_eq!(
        proposed_block.height,
        previous_block_height + 1,
        "Block height is not one greater than previous block height"
    );

    // [3] verify the proposed prev_block_hash matches real previous_block_hash
    assert_eq!(
        proposed_block.prev_blockhash.to_little_endian(),
        previous_block_hash,
        "Proposed prev_block hash does not match real prev_block hash"
    );

    // [4] verify PoW (double sha256(block_hash) <= target)
    assert_pow(&proposed_block_hash, proposed_block, proposed_target);
}

pub fn assert_target_bits(
    last_epoch_begin_block: &Block,
    last_epoch_end_block: &Block,
    new_epoch_begin_block: &Block,
) {
    let old_target_difficulty = bits_to_target(
        last_epoch_begin_block.bits
    );
    let new_target_difficulty = old_target_difficulty
        .checked_mul(&U256::from_u32(
            u32::from_le_bytes(last_epoch_end_block.time) - u32::from_le_bytes(last_epoch_begin_block.time),
        ))
        .unwrap()
        .checked_div(&U256::from_u32(EXPECTED_EPOCH_SECONDS))
        .unwrap();

    let new_bits = u32::from_le_bytes(
        new_epoch_begin_block.bits
    );
    let (mant, expt) = (new_bits >> 24, new_bits & 0xFFFFFF);
    if mant <= 3 {
        assert_eq!(
            new_target_difficulty,
            U256::from_u32(expt) >> (8 * (3 - mant) as usize)
        );
    } else {
        assert_eq!(
            new_target_difficulty >> (8 * (mant - 3) as usize),
            U256::from_u32(expt)
        );
    }
}

pub fn assert_blockchain(
    commited_block_hashes_merkle_root: [u8; 32],
    safe_block_height: u64,
    retarget_block_hash: [u8; 32],
    blocks: Vec<Block>,
    retarget_block: Block,
) {
    // check committed retarget block
    assert_eq!(
        retarget_block.compute_block_hash(),
        retarget_block_hash,
        "Initial Retarget block hash mismatch"
    );

    let mut last_retarget_block = retarget_block;
    let mut block_hashes = vec![];
    // the first block in this array is a safe block aka known to the contract
    for i in 0..blocks.len() - 1 {
        let current_block = &blocks[i];
        let next_block = &blocks[i + 1];
        let current_block_hash = current_block.compute_block_hash();
        let next_block_hash = next_block.compute_block_hash();
        block_hashes.push(current_block_hash);

        // check target bits
        if next_block.height % 2016 == 0 {
            assert_target_bits(&last_retarget_block, current_block, next_block);
            last_retarget_block = *next_block;
        }

        // check block header
        verify_block(
            next_block_hash,
            current_block_hash,
            next_block,
            &last_retarget_block,
            safe_block_height + i as u64,
        );
    }
    block_hashes.push(blocks.last().unwrap().compute_block_hash());

    // check committed merkle root of block hashes
    assert_eq!(
        commited_block_hashes_merkle_root,
        get_merkle_root(block_hashes)
    );
}
