use bitcoin::hashes::Hash;
use bitcoin::Block;
// use crypto_bigint::{Encoding, U256};

use crate::{AsOptimizedBlock};
use zk_light_client_core::sha256_merkle::get_merkle_root;
use zk_light_client_core::{CircuitInput, CircuitPublicValues};
use zk_light_client_core::btc_light_client::AsLittleEndianBytes;

// use sp1_sdk::{ExecutionReport, HashableKey, ProverClient, SP1Stdin};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const MAIN_ELF: &[u8] = include_bytes!("../../elf/riscv32im-succinct-zkvm-elf");

pub fn build_block_proof_input(
    safe_block_height: u64,
    blocks: &[Block],
    retarget_block: &Block,
    retarget_block_height: u64,
) -> CircuitInput {
    // convert standard Block into optimized Block (only contains block header info)
    let optimized_blocks = &blocks
        .iter()
        .zip(safe_block_height..safe_block_height + blocks.len() as u64)
        .map(|(block, height)| block.as_optimized_block(height))
        .collect::<Vec<_>>();

    CircuitInput::new(
        CircuitPublicValues::new(
            retarget_block
                .header
                .block_hash()
                .to_byte_array()
                .to_little_endian(),
            safe_block_height,
            get_merkle_root(
                blocks
                    .iter()
                    .map(|block| block.header.block_hash().to_byte_array().to_little_endian())
                    .collect::<Vec<_>>(),
            ),
        ),
        optimized_blocks.to_vec(),
        retarget_block.as_optimized_block(retarget_block_height),
    )
}
