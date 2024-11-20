pub mod btc_light_client;
pub mod constants;
pub mod sha256_merkle;

use constants::{MAX_BLOCKS};
use alloy_sol_types::sol;
// use crypto_bigint::U256;
use serde::{Deserialize, Serialize};

mod arrays {
    use std::{convert::TryInto, marker::PhantomData};

    use serde::{
        de::{SeqAccess, Visitor},
        ser::SerializeTuple,
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub fn serialize<S: Serializer, T: Serialize, const N: usize>(
        data: &[T; N],
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = ser.serialize_tuple(N)?;
        for item in data {
            s.serialize_element(item)?;
        }
        s.end()
    }

    struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

    impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
    where
        T: Deserialize<'de>,
    {
        type Value = [T; N];

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(&format!("an array of length {}", N))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // can be optimized using MaybeUninit
            let mut data = Vec::with_capacity(N);
            for _ in 0..N {
                match (seq.next_element())? {
                    Some(val) => data.push(val),
                    None => return Err(serde::de::Error::invalid_length(N, &self)),
                }
            }
            match data.try_into() {
                Ok(arr) => Ok(arr),
                Err(_) => unreachable!(),
            }
        }
    }
    pub fn deserialize<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        deserializer.deserialize_tuple(N, ArrayVisitor::<T, N>(PhantomData))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct CircuitPublicValues {
    pub retarget_block_hash: [u8; 32],
    pub safe_block_height: u64,
    pub block_hashes_merkle_root: [u8; 32],
}

sol! {
    struct ProofPublicInputs {
        bytes32 retarget_block_hash;
        uint64 safe_block_height;
        bytes32 block_hashes_merkle_root;
    }
}

impl Default for CircuitPublicValues {
    fn default() -> Self {
        CircuitPublicValues {
            retarget_block_hash: [0u8; 32],
            safe_block_height: 0,
            block_hashes_merkle_root: [0u8; 32],
        }
    }
}

impl CircuitPublicValues {
    pub fn new(
        retarget_block_hash: [u8; 32],
        safe_block_height: u64,
        block_hashes_merkle_root: [u8; 32],
    ) -> Self {
        Self {
            retarget_block_hash,
            safe_block_height,
            block_hashes_merkle_root,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CircuitInput {
    pub public_values: CircuitPublicValues,
    #[serde(with = "arrays")]
    pub blocks: [btc_light_client::Block; MAX_BLOCKS],
    pub utilized_blocks: u64,
    pub retarget_block: btc_light_client::Block,
}

impl CircuitInput {
    pub fn new(
        public_values: CircuitPublicValues,
        blocks: Vec<btc_light_client::Block>,
        retarget_block: btc_light_client::Block,
    ) -> Self {
        let mut padded_blocks = [btc_light_client::Block::default(); MAX_BLOCKS];
        for (i, block) in blocks.iter().enumerate() {
            padded_blocks[i] = *block;
        }

        Self {
            public_values,
            blocks: padded_blocks,
            utilized_blocks: blocks.len() as u64,
            retarget_block,
        }
    }
}

impl Default for CircuitInput {
    fn default() -> Self {
        Self {
            public_values: CircuitPublicValues::default(),
            blocks: [btc_light_client::Block::default(); MAX_BLOCKS],
            utilized_blocks: 0,
            retarget_block: btc_light_client::Block::default(),
        }
    }
}

pub fn validate_block(circuit_input: CircuitInput) -> CircuitPublicValues {
    let blocks = circuit_input.blocks[0..(circuit_input.utilized_blocks as usize)].to_vec();
    // Block Verification
    btc_light_client::assert_blockchain(
        circuit_input.public_values.block_hashes_merkle_root,
        circuit_input.public_values.safe_block_height,
        circuit_input.public_values.retarget_block_hash,
        blocks,
        circuit_input.retarget_block,
    );

    circuit_input.public_values
}
