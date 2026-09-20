# RFC-0010 — VoxCPM2 TTS Engine (Pure Rust via `voxcpm-rs`)

- **Status:** Draft for review
- **Author:** Dani
- **Parent:** [RFC-0001](RFC-0001-desktop-companion.md) (Desktop Companion)
- **Relates to:** [RFC-0005](RFC-0005-voice-output-tts.md) (voice output), [RFC-0009](RFC-0009-privacy-offline.md) (privacy/offline), [RFC-0006](RFC-0006-memory-system.md) (recordings as sensitive data)
- **Covers (PRD):** F-11 (read-back) — an additional engine, not a replacement
- **ACs:** AC-06 (Esc stops speech), AC-09 (offline stays offline) · **Milestone:** post-M3, optional; gated on the spike in §10

---

## 1. Summary

Add a third read-aloud engine — **VoxCPM2 via the `voxcpm-rs` crate** (pure
Rust on the Burn framework) — for expressive prosody and local zero-shot voice
cloning. Kokoro remains the default; system voices remain the always-available
fallback. Because the crate is Rust, the engine runs **in-process** in the
shell: no Python sidecar, no ONNX runtime, no audio transport, no new IPC.

**On the original request (VoxCPM *0.5B*):** `voxcpm-rs` implements
**VoxCPM2 (2B)**, not 0.5B. The only Rust implementation of 0.5B
(`madushan1000/voxcpm_rs`) is **AGPL-3.0**, which is unsuitable for this
codebase. Upstream retired 0.5B in favour of VoxCPM1.5 and VoxCPM2, and only
VoxCPM2 has the ANE/MLX/GGUF ecosystem with real-time-capable Apple numbers.
This RFC therefore targets VoxCPM2, and 0.5B is a non-goal (§3).

## 2. Motivation

- **Expressiveness and cloning.** Kokoro-82M is stable, fast and well-liked,
  but its prosody is flat and it has no voice cloning. VoxCPM2 adds both, in
  zh + en, fully locally.
- **The Rust path is dramatically cheaper than the Python path.** Running the
  reference implementation means `torch` (+~2 GB) inside the sidecar, base64
  PCM over our JSON-RPC line protocol, and a new audio transport to design and
  test. `voxcpm-rs` deletes that entire surface: one cargo dependency, audio
  produced in the same process that plays it.
- **It does not disturb the memory envelope when unused.** RFC-0002's idle
  envelope (147.6 MB measured in S2) survives because the engine is lazy-loaded
  only while selected and unloaded when the user switches away (§4.7).
- **Differentiation is local.** Weights live on disk, inference is offline, and
  no audio or text leaves the machine (RFC-0009).

## 3. Goals / Non-Goals

**Goals**
- A third `tts.engine` option (`voxcpm`) with the existing fallback ladder.
- Chunk-streamed playback with a measured time-to-first-audio, reusing the
  sentence/chunk sink pipeline Kokoro already uses.
- Esc/hide cancels synthesis cooperatively (`CancelToken`, AC-06).
- Model weights fetched through the existing catalog/downloader flow (§4.8).
- Optional cargo feature so default builds stay lean (§4.10).
- Voice cloning from material the user already has — a local reference clip —
  as a clearly opt-in, deletable artifact (§4.6).

**Non-Goals**
- Replacing Kokoro as the default engine or the system-voice fallback.
- Bundling ~4.5 GB of weights in the app bundle or the dmg.
- VoxCPM **0.5B** support (crate does not implement it; the 0.5B Rust
  alternative is AGPL-3.0).
- The `vulkan` backend (bf16, ~2.6× faster on AMD RDNA4) — it requires a
  `[patch.crates-io]` on `burn-cubecl`/`cubecl-spirv` and is verified on AMD
  only; Apple hardware is unproven. Revisit in a later RFC (§7).
- Training, fine-tuning, cloud cloning, or any server-side inference.
- Batch/multi-utterance throughput APIs — a desktop has one speaker (§7).

## 4. Detailed Design

### 4.1 Engine selection and the fallback ladder

`tts.engine` gains a third value. Resolution order stays local and explicit:

| Setting | Behaviour |
|---|---|
| `voxcpm` | VoxCPM2 if weights are present and the model loads; on load failure fall back to `kokoro` and surface why in the log |
| `kokoro` | current default |
| `system` | macOS `say`, always available |

Switching **away** from `voxcpm` releases the model (§4.7). Enabling it in
Settings shows the measured memory cost before the user commits (§4.7).

### 4.2 Crate choice and rejected alternatives

| Option | Verdict |
|---|---|
| **`voxcpm-rs` 0.5.0** (Apache-2.0, Burn 0.20, Rust 1.85+) | **Chosen.** VoxCPM2, features `wgpu` → **Metal on macOS**, `cpu`/`cpu-blas` fallback, `generate_stream`, `CancelToken`, `batch`, voice cloning, safetensors consumed as-shipped |
| Reference Python (PyTorch) | Rejected: ~2 GB `torch`, sidecar audio transport, base64 PCM over JSON-RPC — all avoidable |
| `madushan1000/voxcpm_rs` (0.5B, Rust/Burn) | Rejected: **AGPL-3.0** |
| `bluryar/VoxCPM-ONNX` (0.5B only) | Rejected: archived, 0.5B only, 16 kHz, no maintained path |
| `llama.cpp-omni` subprocess (GGUF, Metal) | Rejected for now: a second external binary; Metal RTF ≈ 1.06 (0.5B) / 1.76 (VoxCPM2) on M4 Pro — slower than real time |

Diligence notes to carry into the spike: crates.io reports **no license field**
for the published crate although the repository ships Apache-2.0 and
`Cargo.toml` declares it — pin the version and vendor the LICENSE. Model-card
terms for `openbmb/VoxCPM2` must be confirmed before shipping weights.

### 4.3 Runtime architecture (in-process, off the UI thread)

```
panel:complete ──► tts::speak_text ──► voxcpm worker thread
                                          │  Arc<Mutex<VoxCPM<B>>>   (model is !Sync)
                                          │  generate_stream(text, opts)
                                          ▼
                                     chunks: Vec<f32> @ model.sample_rate()
                                          │
                                          ▼
                                     rodio sink (existing pipeline)
```

- `VoxCPM<B>` is **not `Sync`** (Burn's `Param` holds a `OnceCell`), so the
  model lives behind `Arc<Mutex<…>>` and all generation is serialized on a
  **dedicated worker thread**. Concurrent read-aloud requests queue; a newer
  answer drops the stale one (existing generation-counter behaviour).
- Load is lazy and expensive: `from_local` takes **~20–25 s** for the 4.3 GB
  BF16 backbone (upcast to F32 on the `wgpu` backend, because WGSL has no BF16
  type). The load is prewarmed when the user selects the engine, and its
  progress is logged (§4.4).
- The Burn/`log` output is bridged to our `ruoxi:` stderr lines so the dev
  console keeps one vocabulary.

### 4.4 Streaming and playback integration

`VoxCPM::generate_stream(text, opts)` yields `Result<Vec<f32>>` chunks; the
AudioVAE decoder is causal, so concatenated chunks equal `generate()` exactly.
`chunk_patches(5)` is ~400 ms per chunk at the default config — the same
order as our current sentence chunking, so the existing single-sink pipeline
(`speak_kokoro_chunked`: one `rodio::Player`, sequential `append`, generation
check between chunks) generalizes to an engine-agnostic chunk feeder rather
than being duplicated.

Logging per utterance: model load ms (once), first-chunk ms, per-chunk synth
ms and audio seconds — the same shape as the Kokoro timing lines, so the
RFC-0005 latency table stays comparable.

### 4.5 Cancellation (AC-06)

`CancelToken` is `Clone + Send + Sync` (an `Arc<AtomicBool>`), polled between
diffusion steps; cancel latency is bounded by **one step (~200 ms at
`timesteps = 10` on `wgpu`)**. Our `SPEAK_GEN` atomic maps directly onto it:
a new answer, Esc, or panel hide cancels the in-flight generation instead of
letting it burn a GPU pass nobody will hear.

### 4.6 Voice cloning (opt-in, local, deletable)

`Prompt::Reference { audio }` plus `PromptAudio::Pcm { samples, sample_rate }`
means a reference voice can come from **raw samples we already have** (a PTT
recording) or an imported clip; `Prompt::Continuation` continues an existing
utterance. Rules:

- Cloning is explicitly user-initiated; the reference clip is a local file the
  user can delete from Settings, and it never leaves the machine (RFC-0009).
- The reference clip is resampled to the model's expected rate on use.
- No cloning is inferred from capture content — this is a deliberate action,
  consistent with the poisoning rules in RFC-0006 §4.9.

### 4.7 Memory and the envelope

The decisive constraint. On the `wgpu` backend the 4.3 GB BF16 backbone is
upcast to F32 → **≈8.6 GB of weights**, plus AudioVAE and activations. On a
16 GB machine (the current dev target) that is workable but heavy, and it is
the one number that decides this RFC (§10).

Mitigations, all required:
- Never loaded at startup; only while `voxcpm` is the selected engine.
- Unloaded on engine switch, and after an idle timeout (default TBD, §7).
- Settings shows the measured peak before enabling, phrased as a trade-off.
- If allocation or load fails, fall back to Kokoro and log the reason.

### 4.8 Model artifacts and download

`VoxCPM::from_local` consumes an HF directory **as shipped** — no conversion:

| File | Size (approx.) |
|---|---|
| `config.json`, `tokenizer.json` | small |
| `model.safetensors` (LM + DiT, BF16) | ~4.3 GB |
| `audiovae.pth` / `audiovae.safetensors` | VAE decoder |

Our downloader currently fetches single files; this needs a **multi-file
catalog entry** (directory layout, per-file checksum, resumable download,
aggregate progress on the existing `model:progress` channel) plus a size
warning and a delete action. Download is ~4.5 GB — the largest artifact the
app ships or fetches, and the first one that is genuinely optional.

### 4.9 Platform matrix

| Platform | Backend | Status |
|---|---|---|
| macOS (arm64) | `wgpu` → Metal | primary target; **unmeasured** (§10) |
| macOS CPU-only | `cpu-blas` (vendored OpenBLAS) | correctness fallback; expected far slower than real time |
| Windows | `wgpu` → DX12 | follows RFC-0002's Windows track |
| Linux | `wgpu` → Vulkan | untested |
| any + `vulkan` feature | native Vulkan, bf16 | deferred (§3) |

### 4.10 Feature gating and build cost

The engine sits behind an optional cargo feature (default **off**) so the
standard build keeps its current dependency graph and compile time; Burn 0.20
plus `wgpu`/`cubecl` is a substantial addition. The release profile can enable
it; CI keeps default features so a broken optional path cannot block the tree.

## 5. Interfaces & Data Structures

```jsonc
// settings.json (existing shape, one new value + clone ref)
{
  "tts": {
    "engine": "kokoro",            // "system" | "kokoro" | "voxcpm"
    "voice": "af_heart",
    "model": "",                    // kokoro model id (unchanged)
    "voxcpm_model_dir": "",         // null until downloaded
    "clone_reference": ""            // local clip path; empty = no cloning
  }
}
```

- Rust: `tts::speak_text` gains the `voxcpm` arm; a dedicated worker owns
  `Arc<Mutex<VoxCPM<B>>>`; `SPEAK_GEN` → `CancelToken`.
- Commands: reuse `tts_download_kokoro`-style flow for the new entry
  (`tts_download_voxcpm`, or a generalized multi-file download).
- Events: existing `model:progress` / `model:done` for the download; existing
  panel states (`speaking`) for playback. No new event names.
- Log lines: `ruoxi: tts engine voxcpm ready in <ms>`, `tts first audio after
  <ms>`, per-chunk `chars/ms/audio` — same shape as Kokoro's.

## 6. Risks & Failure Modes

| Risk | Impact | Mitigation |
|---|---|---|
| Peak RAM ≈ 9–10 GB on a 16 GB machine | OOM or swap thrash while the user works | Spike gate (§10); lazy load; unload on switch/idle; Settings shows measured cost; auto-fallback to Kokoro |
| RTF ≥ 1 on Apple GPUs | Read-aloud slower than speech — unusable | All published numbers are AMD RX 9070 XT (bf16) or MLX; **measure on M4 Pro**. Gate: RTF < 0.6 with `chunk_patches(3–5)` |
| 20–25 s cold load | Feels broken on first use | Prewarm on engine select, progress logged and surfaced |
| `!Sync` model + mutex | Concurrent asks serialize/queue | Single TTS worker; drop-stale generation counter (already the pattern) |
| crates.io shows no license; model terms unconfirmed | Compliance surprise | Pin exact version, vendor LICENSE, confirm model card before shipping |
| Sample-rate ambiguity (crate's architecture table says 16 kHz AudioVAE; upstream markets 48 kHz) | Wrong resampling → pitch/speed artefacts | Always read `model.sample_rate()`; verify in spike; resample at the sink |
| 4.5 GB download and disk footprint | User surprise, failed downloads | Size warning, resumable multi-file download, delete action |
| Crate maturity (0.5.0, partially documented, API churn) | Breakage on upgrade | Pin version; isolate behind our own engine trait; upgrade deliberately |
| Quality regression vs Kokoro on zh/en | Users disable it | A/B in the spike; Kokoro stays default regardless |

## 7. Open Questions

1. **Does it ship?** RTF and peak RSS on M4 Pro 16 GB — the spike decides
   between "optional engine", "CPU-only experiment", or "defer".
2. Can the F32 upcast be avoided on Apple (Metal-native bf16, or wgpu
   `shader-f16` once available)? Halving weights would change the verdict on
   16 GB machines.
3. Exact VoxCPM2 weight licence/commercial terms, verified at spike time.
4. Is 16 GB enough with the user's other apps, or is this a 32 GB-class
   feature we document as such?
5. Cloning source: reuse a PTT recording, or require an imported clip? What
   consent affordance accompanies "make Ruòxī sound like me"?
6. Idle-unload policy: minutes, or "immediately when the panel hides"?
7. Do we ever need `batch`/`parallel_segments` on desktop, or only if a
   server-side product appears?

## 8. Dependencies

- **RFC-0005** — engine selection, fallback ladder, streaming playback, the
  latency table this RFC must feed.
- **RFC-0009** — offline guarantees, clamping the engine to local inference;
  privacy posture for cloning references.
- **RFC-0006** — recordings/clips are sensitive user data (deletable, local).
- **Model catalog** (`shell/src-tauri/src/models.rs`) — multi-file download
  support is a prerequisite, not part of this RFC's scope but on its critical
  path.
- **`voxcpm-rs` 0.5.0** + Burn 0.20 — pinned; optional cargo feature.

## 9. Acceptance Mapping

| AC | Mechanism |
|---|---|
| AC-06 (Esc stops speech) | `CancelToken` polled between diffusion steps; ≤ ~200 ms |
| AC-09 (offline) | Weights and inference are local; no network during synthesis |
| F-11 (read-back) | Third engine under the same toggle, voice picker and rate control |
| PRD §12 (trust, not surveillance) | Explicit download, local weights, deletable clone reference, no capture-derived cloning |

## 10. Spike gate (before any scheduling)

Protocol on the reference machine (M4 Pro, 16 GB, macOS 26):

1. Build `voxcpm-rs` with `features = ["wgpu"]`; load `openbmb/VoxCPM2` from
   local disk. Record **load ms** and **peak RSS** (the RFC-0002 method).
2. Synthesize a fixed zh + en corpus (the S5 utterance set, reused) at
   `chunk_patches` ∈ {3, 5, 8}, `timesteps` ∈ {7, 10}. Record per-chunk
   latency, **TTFB**, end-to-end **RTF**, and the sample rate reported by
   `model.sample_rate()`.
3. Repeat under memory pressure (a browser plus the dev toolchain running) —
   i.e. the user's real desk, not an idle machine.
4. A/B the zh and en output against the existing Kokoro path.

**Gate:** ship as an optional engine only if **RTF < 0.6**, **TTFB < 800 ms**,
and **peak RSS ≤ 6 GB**. Otherwise record the numbers here, keep Kokoro as the
sole neural engine, and revisit when a bf16 Apple path exists (§7.2).
