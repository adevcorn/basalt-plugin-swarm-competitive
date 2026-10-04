# basalt-plugin-swarm-competitive

Reference Swarm Strategy Plugin for [Basalt](https://github.com/adevcorn/basalt) implementing **Speculative Competitive Racing** with Semantic Telemetry and Category-Based Tool Scoping.

---

## 🚀 Capabilities & Metadata

- **Plugin Name**: `swarm_competitive`
- **Hook Flags**: `CAP_SWARM_STRATEGY` (`0x00000080`)
- **Provides**: `swarm-strategy@competitive/v1`
- **ABI Version**: `1`
- **Target**: `wasm32-unknown-unknown` (WASM sandbox) or native cdylib

---

## ✨ Features

- **Divergent Peer Formulation (`basalt_swarm_formulate_hypotheses`)**:
  - Dynamically synthesizes 4 distinct peer hypotheses from a task prompt:
    - **`implementer`**: Direct, high-confidence primary implementation path (`["read", "write", "verify"]`).
    - **`optimizer`**: Performance, memory, and minimal-overhead focused variant (`["read", "write", "verify"]`).
    - **`adversarial_fuzzer`**: Edge-case and failure-mode discovery path (`["read", "write", "verify"]`).
    - **`architect`**: High-level structural and design review path (`["read", "verify", "peer"]`).
- **Category-Based Tool Scoping**:
  - Assigns explicit tool categories (`tool_categories`) to each peer variant to restrict tool availability dynamically in shadow sessions.
- **Candidate Scoring & Ranking (`basalt_swarm_evaluate_candidates`)**:
  - Multi-criteria weighted scoring matrix:
    - Verification score ($40\%$)
    - Semantic telemetry facts generated ($25\%$)
    - Change compactness / AST delta sanity ($20\%$)
    - Latency / execution speed ($15\%$)
- **Synthesis Recommendation (`basalt_swarm_synthesize`)**:
  - Selects the clear winner if score $\ge 0.85$ or margin $\ge 0.20$.
  - Recommends patch combination/cherry-picking if complementary edits exist.
  - Recommends human review if scores are ambiguous.

---

## 🏗️ Building & Testing

### Build WebAssembly Plugin

```bash
cargo build --target wasm32-unknown-unknown --release
```

Output binary:
`target/wasm32-unknown-unknown/release/basalt_plugin_swarm_competitive.wasm`

### Install into Basalt Plugin Directory

```bash
cp target/wasm32-unknown-unknown/release/basalt_plugin_swarm_competitive.wasm ~/.config/basalt/plugins/swarm_competitive.wasm
```

### Run Unit Tests

```bash
cargo test
```

---

## 📄 License

Licensed under the Apache-2.0 / MIT licenses to match the Basalt ecosystem.
