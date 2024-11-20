pub mod proof;

use bitcoin::hashes::hex::FromHex;
use bitcoin::hashes::Hash;
use std::fmt::Write;

use zk_light_client_core::btc_light_client::Block as OptimizedBlock;

pub fn load_hex_bytes(file: &str) -> Vec<u8> {
    let hex_string = std::fs::read_to_string(file).expect("Failed to read file");
    Vec::<u8>::from_hex(&hex_string).expect("Failed to parse hex")
}

pub fn to_hex_string(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

pub fn get_retarget_height_from_block_height(block_height: u64) -> u64 {
    block_height - (block_height % 2016)
}

pub trait AsOptimizedBlock {
    fn as_optimized_block(&self, height: u64) -> OptimizedBlock;
    fn as_optimized_block_unsafe(&self) -> OptimizedBlock;
}

impl AsOptimizedBlock for bitcoin::Block {
    fn as_optimized_block(&self, height: u64) -> OptimizedBlock {
        OptimizedBlock {
            height,
            version: self.header.version.to_consensus().to_le_bytes(),
            prev_blockhash: self.header.prev_blockhash.to_raw_hash().to_byte_array(),
            merkle_root: self.header.merkle_root.to_raw_hash().to_byte_array(),
            time: self.header.time.to_le_bytes(),
            bits: self.header.bits.to_consensus().to_le_bytes(),
            nonce: self.header.nonce.to_le_bytes(),
        }
    }

    fn as_optimized_block_unsafe(&self) -> OptimizedBlock {
        OptimizedBlock {
            height: self.bip34_block_height().unwrap(),
            version: self.header.version.to_consensus().to_le_bytes(),
            prev_blockhash: self.header.prev_blockhash.to_raw_hash().to_byte_array(),
            merkle_root: self.header.merkle_root.to_raw_hash().to_byte_array(),
            time: self.header.time.to_le_bytes(),
            bits: self.header.bits.to_consensus().to_le_bytes(),
            nonce: self.header.nonce.to_le_bytes(),
        }
    }
}
