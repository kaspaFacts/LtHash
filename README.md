# LtHash (Rust Port)

This is a Rust port of the LtHash algorithm originally implemented in Go.

Original implementations:
- https://pkg.go.dev/lukechampine.com/lthash  
- https://github.com/lukechampine/lthash  

LtHash is based on:

- Bellare & Micciancio: “Incremental Hashing Modulo a Prime”
  https://cseweb.ucsd.edu/~daniele/papers/IncHash.pdf  
- Facebook specification:
  https://eprint.iacr.org/2019/227.pdf  

---

## ⚠️ Status

**This implementation is experimental and untested in production.**

- No formal verification has been performed
- Cross-language compatibility with the Go reference is not guaranteed
- This implementation has not been independently audited and should not be considered cryptographically secure or production-ready
- Behavior may change as the implementation is refined

Use at your own risk.

---

## 🧠 What is LtHash?

LtHash is a **multiset hash function**, meaning:

- Order of inputs does not matter
- Elements can be added and removed incrementally
- The same multiset produces the same hash

It is designed for use cases such as:
- Set reconciliation
- Synchronization protocols
- Probabilistic data comparison

It is **not a general-purpose cryptographic hash function**.

---

## ⚙️ Implementation Overview

This Rust port follows the same conceptual structure as the Go implementation:

- Each input element is expanded into a fixed-length byte vector (2048 bytes)
- Expansion is performed using a BLAKE2b-based extendable-output (XOF-like) construction
- Elements are combined using modular arithmetic over 16-bit words

Core operations:

- `Add(p)` → incorporates an element into the hash
- `Remove(p)` → removes an element from the hash
- `Sum()` → returns the current hash state
- `SetState()` → restores a previous state

---

## 🔐 Hash Construction

Each input element is transformed into a 2048-byte vector using a BLAKE2b-based expansion function.

These vectors are then combined using:

- Little-endian 16-bit modular addition
- Little-endian 16-bit modular subtraction

This produces an algebraic multiset accumulator supporting additive cancellation.

---

## ⚠️ Multiset Collision Warning

LtHash is vulnerable to multiset collisions.

In particular, for `lthash16`, adding the same element approximately 2^16 times will result in a collision due to modular wraparound.

This is a known property of the construction, not a bug.

### Mitigation

To reduce collision risk:

- Append unique metadata to each element (e.g., index, nonce, or identifier)
- Avoid repeated identical inputs without distinguishing context

---

## 🚧 Compatibility Notes

This implementation uses a Rust-based BLAKE2X-style expansion layer built on top of `blake2b_simd`.

Important notes:

- It is intended to approximate Go’s `blake2b.NewXOF` behavior
- However, exact bit-for-bit compatibility with the Go implementation is **not guaranteed**
- Differences in XOF construction details may result in divergent outputs

If strict cross-language determinism is required, additional validation against the Go reference implementation is necessary.

---

## 📦 Usage Example

```rust
use lthash::Hash16;

fn main() {
    let mut h = Hash16::new();

    h.add(b"Hello");
    h.add(b"World");

    let digest = h.sum();

    println!("{:x?}", digest);
}
