#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::private::{FixedBytes};
use alloy_sol_types::SolType;
use zk_light_client_core::{validate_block, CircuitInput, ProofPublicInputs};

pub fn main() {
    // Read an input to the program.
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    let circuit_input = sp1_zkvm::io::read::<CircuitInput>();

    // circuit logic
    let circuit_public_input = validate_block(circuit_input);

    // Encode the public values of the program.
    let bytes = ProofPublicInputs::abi_encode(&ProofPublicInputs {
        retarget_block_hash: FixedBytes::from(circuit_public_input.retarget_block_hash),
        safe_block_height: circuit_public_input.safe_block_height,
        block_hashes_merkle_root: FixedBytes::from(circuit_public_input.block_hashes_merkle_root),
    });

    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    sp1_zkvm::io::commit_slice(&bytes);
}
