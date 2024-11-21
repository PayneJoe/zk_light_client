#[cfg(test)]
mod test {
    use bitcoin::consensus::encode::deserialize;
    use bitcoin::hashes::hex::{FromHex};
    use bitcoin::Block;
    use crypto_bigint::U256;
    use crypto_bigint::{CheckedMul};

    const EPOCH_BLOCK_NUMBER: u32 = 2016;
    const BLOCK_TIMEVAL: u32 = 600;
    const EXPECTED_EPOCH_SECONDS: u32 = EPOCH_BLOCK_NUMBER * BLOCK_TIMEVAL;

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

    pub fn load_hex_bytes(file: &str) -> Vec<u8> {
        let hex_string = std::fs::read_to_string(file).expect("Failed to read file");
        Vec::<u8>::from_hex(&hex_string).expect("Failed to parse hex")
    }

    fn encode_target_to_compact(target: &str) -> u32 {
        // Parse the target as a hexadecimal string into a big integer
        let target_bytes = hex::decode(target).expect("Invalid hex string");

        // Remove leading zeros
        let mut normalized_target = target_bytes.iter().skip_while(|&&byte| byte == 0).cloned().collect::<Vec<u8>>();

        // Determine the exponent (length of the normalized target in bytes)
        let exponent = normalized_target.len() as u8;

        // Ensure the coefficient is 3 bytes (24 bits)
        while normalized_target.len() > 3 {
            normalized_target.pop(); // Keep only the most significant 3 bytes
        }
        while normalized_target.len() < 3 {
            normalized_target.insert(0, 0); // Pad with zeros if less than 3 bytes
        }

        // Combine the exponent and coefficient into the compact representation
        let coefficient = ((normalized_target[0] as u32) << 16)
            | ((normalized_target[1] as u32) << 8)
            | (normalized_target[2] as u32);

        // Compact representation: 1 byte exponent + 3 bytes coefficient
        ((exponent as u32) << 24) | coefficient
    }

    #[test]
    fn test_variant_target_bits() {
        let block_data_path = "/root/blocks/data";

        let last_epoch_begin = 852768;
        let last_epoch_end = last_epoch_begin + EPOCH_BLOCK_NUMBER - 1;
        let new_epoch_begin = last_epoch_begin + EPOCH_BLOCK_NUMBER;
        let last_epoch_begin_block = deserialize::<Block>(&load_hex_bytes(
            format!("{block_data_path}/block_{last_epoch_begin}.hex").as_str(),
        ))
        .unwrap();
        let last_epoch_end_block = deserialize::<Block>(&load_hex_bytes(
            format!("{block_data_path}/block_{last_epoch_end}.hex").as_str(),
        ))
        .unwrap();

        let new_epoch_begin_block = deserialize::<Block>(&load_hex_bytes(
            format!("{block_data_path}/block_{new_epoch_begin}.hex").as_str(),
        ))
        .unwrap();

        let old_target_difficulty = bits_to_target(last_epoch_begin_block.header.bits.to_consensus().to_le_bytes());
        let new_target_difficulty = old_target_difficulty.checked_mul(&U256::from_u32(last_epoch_end_block.header.time - last_epoch_begin_block.header.time)).unwrap().checked_div(&U256::from_u32(EXPECTED_EPOCH_SECONDS)).unwrap();

        let new_bits = u32::from_le_bytes(new_epoch_begin_block.header.bits.to_consensus().to_le_bytes());
        let (mant, expt) = (new_bits >> 24, new_bits & 0xFFFFFF);
        if mant <= 3 {
            assert_eq!(new_target_difficulty, U256::from_u32(expt) >> (8 * (3 - mant) as usize));
        } else {
            assert_eq!(new_target_difficulty >> (8 * (mant - 3) as usize), U256::from_u32(expt));
        }

        println!(
            "{} = {} * {} = {}",
            new_target_difficulty,
            old_target_difficulty,
            (last_epoch_end_block.header.time - last_epoch_begin_block.header.time) as f32 * 1.0 / EXPECTED_EPOCH_SECONDS as f32,
            bits_to_target(new_epoch_begin_block.header.bits.to_consensus().to_le_bytes())
        );
    }

    #[test]
    fn test_target_encode() {
        // Example target
        let target = "000000000000000000031ABEE416C16C16C16C16C16C16C16C16C16C16C16C16";

        // Encode to compact representation
        let compact = encode_target_to_compact(target);

        // Print the result
        println!("Target: {}", target);
        println!("Compact Representation (bits): 0x{:08X}", compact);
    }
}
