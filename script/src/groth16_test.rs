use bitcoin::consensus::encode::deserialize;

use bitcoin::Block;

use zk_light_client_core::CircuitInput;
use zk_light_client_lib::proof::{self, build_block_proof_input};
use zk_light_client_lib::{get_retarget_height_from_block_height, load_hex_bytes, to_hex_string};

use clap::Parser;
use sp1_sdk::{ProverClient, SP1Stdin};

fn get_test_case_circuit_input() -> CircuitInput {
    let num_blocks = 500;
    let mined_blocks = (854373..(854373 + num_blocks)).map(|height| deserialize::<Block>(&load_hex_bytes(format!("../tests/data/block_{height}.hex").as_str())).unwrap()).collect::<Vec<_>>();

    let mined_block_height = 854374;
    let retarget_block_height = get_retarget_height_from_block_height(mined_block_height);
    let mined_retarget_block = deserialize::<Block>(&load_hex_bytes(
        format!("../tests/data/block_{retarget_block_height}.hex").as_str(),
    ))
    .unwrap();

    build_block_proof_input(
        mined_blocks.first().unwrap().bip34_block_height().unwrap(),
        mined_blocks.as_slice(),
        &mined_retarget_block,
        retarget_block_height,
    )
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run only the execute block without proof generation
    #[arg(long)]
    execute: bool,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse command line arguments
    let args = Args::parse();

    let circuit_input = get_test_case_circuit_input();

    println!("Circuit input generated successfully.");

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();

    stdin.write(&circuit_input);
    println!("Inputs serialized successfully.");

    if args.execute {
        // Execute the program
        let (_output, report) = client.execute(proof::MAIN_ELF, stdin).run().unwrap();
        println!("Program executed successfully.");
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(proof::MAIN_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, stdin)
            .groth16()
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
        println!(
            "Public Inputs: {:?}",
            to_hex_string(proof.public_values.to_vec().as_slice())
        );
        println!("Solidity Ready Proof: {:?}", to_hex_string(&proof.bytes()));
    }
}

