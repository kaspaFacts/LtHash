#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn fuzz_lthash_properties() {
        let mut rng = rand::thread_rng();

        // number of fuzz iterations
        const ITERS: usize = 1000;

        for _ in 0..ITERS {
            // === Generate random inputs ===
            let mut rand_bytes = |max_len: usize| {
                let len = rng.gen_range(0..max_len);
                let mut v = vec![0u8; len];
                rng.fill(&mut v[..]);
                v
            };

            let x = rand_bytes(256);
            let y = rand_bytes(256);
            let z = rand_bytes(256);

            // === 1. Determinism (blake2xb) ===
            {
                let mut a = [0u8; 2048];
                let mut b = [0u8; 2048];

                blake2xb(&x, &mut a);
                blake2xb(&x, &mut b);

                assert_eq!(a, b, "blake2xb is not deterministic");
            }

            // === 2. Add/remove cancel ===
            {
                let mut h = Hash16::new();
                let before = h.sum();

                h.add(&x);
                h.remove(&x);

                assert_eq!(h.sum(), before, "add/remove did not cancel");
            }

            // === 3. Commutativity ===
            {
                let mut a = Hash16::new();
                let mut b = Hash16::new();

                a.add(&x);
                a.add(&y);

                b.add(&y);
                b.add(&x);

                assert_eq!(a.sum(), b.sum(), "not commutative");
            }

            // === 4. Associativity ===
            {
                let mut h1 = Hash16::new();
                let mut h2 = Hash16::new();

                h1.add(&x);
                h1.add(&y);
                h1.add(&z);

                h2.add(&x);
                h2.add(&y);
                h2.add(&z);

                assert_eq!(h1.sum(), h2.sum(), "not associative");
            }

            // === 5. State roundtrip ===
            {
                let mut h1 = Hash16::new();

                h1.add(&x);
                h1.add(&y);

                let state = h1.sum();

                let mut h2 = Hash16::new();
                h2.set_state(&state);

                assert_eq!(h1.sum(), h2.sum(), "state roundtrip failed");
            }

            // === 6. Avalanche sanity ===
            {
                let mut a = [0u8; 2048];
                let mut b = [0u8; 2048];

                let mut x2 = x.clone();
                if !x2.is_empty() {
                    x2[0] ^= 0x01; // flip one bit
                }

                blake2xb(&x, &mut a);
                blake2xb(&x2, &mut b);

                assert_ne!(a, b, "no avalanche effect");
            }

            // === 7. Random sequence invariants ===
            {
                let mut h = Hash16::new();
                let mut history = Vec::new();

                let ops = rng.gen_range(1..50);

                for _ in 0..ops {
                    let data = rand_bytes(128);

                    if rng.gen_bool(0.5) {
                        h.add(&data);
                        history.push(("add", data));
                    } else {
                        h.remove(&data);
                        history.push(("remove", data));
                    }
                }

                // Replay from scratch
                let mut h2 = Hash16::new();

                for (op, data) in &history {
                    match *op {
                        "add" => h2.add(data),
                        "remove" => h2.remove(data),
                        _ => unreachable!(),
                    }
                }

                assert_eq!(h.sum(), h2.sum(), "non-deterministic history replay");
            }
        }
    }
}
