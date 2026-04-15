use blake2b_simd::Params;

const SIZE: usize = 2048;
const BLOCK_SIZE: usize = 64;

/// Full-fidelity BLAKE2Xb (matches Go's blake2b.NewXOF)
fn blake2xb(input: &[u8], out: &mut [u8]) {
    // === Phase 1: root ===
    let root = Params::new()
        .hash_length(64)
        .fanout(1)
        .max_depth(2)
        .leaf_length(0)
        .node_offset(0)
        .node_depth(0)
        .inner_hash_length(64)
        .last_node(true)
        .to_state()
        .update(input)
        .finalize();

    let root_bytes = root.as_bytes();

    // === Phase 2: expansion ===
    let mut offset = 0;
    let mut node_offset = 0u64;

    let total_blocks = (out.len() + BLOCK_SIZE - 1) / BLOCK_SIZE;

    while offset < out.len() {
        let remaining = out.len() - offset;
        let block_len = remaining.min(BLOCK_SIZE);

        let is_last = node_offset == (total_blocks as u64 - 1);

        let hash = Params::new()
            .hash_length(block_len)
            .fanout(1)
            .max_depth(2)
            .leaf_length(0)
            .node_offset(node_offset)
            .node_depth(1)
            .inner_hash_length(64)
            .last_node(is_last)
            .to_state()
            .update(root_bytes)
            .finalize();

        out[offset..offset + block_len]
            .copy_from_slice(&hash.as_bytes()[..block_len]);

        offset += block_len;
        node_offset += 1;
    }
}

/// LtHash equivalent to Go's hash16
pub struct Hash16 {
    sum: [u8; SIZE],
    hbuf: [u8; SIZE],
}

impl Hash16 {
    /// Equivalent to New16()
    pub fn new() -> Self {
        Self {
            sum: [0u8; SIZE],
            hbuf: [0u8; SIZE],
        }
    }

    /// hashObject equivalent
    fn hash_object(&mut self, p: &[u8]) -> &[u8; SIZE] {
        blake2xb(p, &mut self.hbuf);
        &self.hbuf
    }

    /// Add implements Hash.Add
    pub fn add(&mut self, p: &[u8]) {
        let h = *self.hash_object(p);
        add16(&mut self.sum, &h);
    }

    /// Remove implements Hash.Remove
    pub fn remove(&mut self, p: &[u8]) {
        let h = *self.hash_object(p);
        sub16(&mut self.sum, &h);
    }

    /// Sum implements Hash.Sum
    pub fn sum(&self) -> Vec<u8> {
        self.sum.to_vec()
    }

    /// Equivalent to Go's Sum(b []byte)
    pub fn sum_into(&self, mut b: Vec<u8>) -> Vec<u8> {
        b.extend_from_slice(&self.sum);
        b
    }

    /// SetState implements Hash.SetState
    pub fn set_state(&mut self, state: &[u8]) {
        self.sum.fill(0);
        let len = state.len().min(SIZE);
        self.sum[..len].copy_from_slice(&state[..len]);
    }
}

/// add16: little-endian u16 wrapping addition
fn add16(x: &mut [u8; SIZE], y: &[u8; SIZE]) {
    for i in (0..SIZE).step_by(2) {
        let xi = u16::from_le_bytes([x[i], x[i + 1]]);
        let yi = u16::from_le_bytes([y[i], y[i + 1]]);
        let sum = xi.wrapping_add(yi);
        let bytes = sum.to_le_bytes();
        x[i] = bytes[0];
        x[i + 1] = bytes[1];
    }
}

/// sub16: little-endian u16 wrapping subtraction
fn sub16(x: &mut [u8; SIZE], y: &[u8; SIZE]) {
    for i in (0..SIZE).step_by(2) {
        let xi = u16::from_le_bytes([x[i], x[i + 1]]);
        let yi = u16::from_le_bytes([y[i], y[i + 1]]);
        let sum = xi.wrapping_sub(yi);
        let bytes = sum.to_le_bytes();
        x[i] = bytes[0];
        x[i + 1] = bytes[1];
    }
}
