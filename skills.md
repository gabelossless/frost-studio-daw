# 🧠 Agent Fleet Skills Guideline (`skills.md`)

This guide outlines the specialized skills, safety rules, and performance boundaries for the **3 New Senior Nodes** allocated to finalize absolute production readiness on Frost Studio.

---

## 🔬 1. Senior Software Engineer (Rust / C++)
*   **Domain**: Low-latency bounds memory allocations & Mutex lock reduction.
*   **Core Standards**:
    1.  **Zero-Allocation Pipeline**: The audio processor hotpaths (`process()`, `generate_frame()`) must NEVER trigger `ALLOC` calls on the heap.
        *   ❌ Avoid: `Vec::new()`, `String::from()`, `Box::new()`, `.clone()` on dynamic structures.
        *   ✅ Use: pre-allocated ring buffers, static lookup vectors constants.
    2.  **Lock-Free Concurrency**:
        *   ❌ Avoid: standard `std::sync::Mutex` inside audio iterations (creates Priority Inversion clicks).
        *   ✅ Use: Atomic primitives (`AtomicBool`, `AtomicF32`), crossbeam queues or `ArcSwap`.

---

## 🔬 2. Audio DSP Researcher
*   **Domain**: Advanced filtering pipelines, waveshaping mathematics layout scaling.
*   **Core Standards**:
    1.  **Fast Approximations**:
        *   Static trigonometric branches (`atan()`, `sin()`, `log()`) are CPU expensive in high-volume buffer streams.
        *   Use static Lookup Tables (LUTs) or padè polynomial approximations where 99% precision is sufficient.
    2.  **Gain-Staging Compliance**:
        *   Guarantee parameter coefficient sweeps smoothly interpolate frames to avoid automation arithmetic Pops and Click outputs securely.

---

## 🖥️ 3. Core Frontend Architect (React)
*   **Domain**: Framerate stability, Canvas buffers frame thresholds optimizations.
*   **Core Standards**:
    1.  **Granular Zustand Selectors**:
        *   ❌ Avoid: `const { meters } = useDawStore()` (forces rerender on ANY state mutation inside store).
        *   ✅ Use: `const meters = useDawStore(state => state.meters)` (only triggers rerender if meters truly update).
    2.  **RequestAnimationFrame (rAF) loops**:
        *   Always use canvas buffers for visualizers streams instead of state triggers updates for direct display paints absolute lagless accurately.

---

## 🚀 Execution Targets:
- **Senior SWE**: Run `cargo expand` or read `engine.rs` looking for allocation hotpaths.
- **DSP Researcher**: Propose pre-calculated scale branches for Eq bounds.
- **Frontend Architect**: Profile memory usage streams.
