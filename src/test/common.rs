pub struct PseudoRandom {
    state: u64,
}

impl PseudoRandom {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    pub fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn fill_buffer(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let val = self.next().to_le_bytes();
            let len = chunk.len();
            chunk.copy_from_slice(&val[..len]);
        }
    }
}

pub fn generate_test_block(block_number: u64, block_size: usize) -> Vec<u8> {
    let mut data = vec![0u8; block_size];
    // First 8 bytes = block number
    data[..8].copy_from_slice(&block_number.to_le_bytes());
    // Rest = pseudo-random based on block number
    let mut rng = PseudoRandom::new(block_number.wrapping_add(0x1234567890ABCDEF));
    rng.fill_buffer(&mut data[8..]);
    data
}

pub enum BlockVerifyResult {
    Ok,
    WrongMapping { expected: u64, got: u64 },
    DataCorruption,
    Error,
}

pub fn verify_test_block(data: &[u8], expected_block: u64) -> BlockVerifyResult {
    if data.len() < 8 {
        return BlockVerifyResult::Error;
    }
    let stored_block = u64::from_le_bytes(data[..8].try_into().unwrap());
    if stored_block != expected_block {
        return BlockVerifyResult::WrongMapping {
            expected: expected_block,
            got: stored_block,
        };
    }
    let expected_data = generate_test_block(expected_block, data.len());
    if data != expected_data.as_slice() {
        return BlockVerifyResult::DataCorruption;
    }
    BlockVerifyResult::Ok
}

pub fn should_stop() -> bool {
    crate::STOP_FLAG.load(std::sync::atomic::Ordering::Relaxed)
}
