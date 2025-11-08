# Platform Support - Cross-Platform GPU Detection & PostgreSQL Storage

**Last Updated**: 2025-01 (Phase 13b.17)
**Phase**: 13b.17 (PostgreSQL Infrastructure + GPU Detection)
**Status**: Validated on macOS + Linux, PostgreSQL tested on macOS/Linux/Docker

## Overview

rs-llmspell supports cross-platform GPU acceleration with platform-aware device detection and graceful fallback strategies. The system automatically selects the best available compute device while ensuring zero compilation errors across all platforms.

## Supported Platforms

| Platform | Primary GPU | Fallback | Status |
|----------|-------------|----------|--------|
| macOS (Apple Silicon) | Metal | CPU | ✅ Validated |
| macOS (Intel) | CPU | - | ✅ Validated |
| Linux | CUDA | CPU | ✅ Validated |
| Windows | CUDA | CPU | 🔄 Expected Compatible |

## GPU Detection Logic

### Device Selection Modes

The Candle provider supports four device selection modes via the `device` parameter:

```rust
CandleProvider::new(
    model_name,
    model_directory,
    device_mode  // "auto", "cpu", "cuda", or "metal"
)
```

#### 1. Auto Mode (`"auto"`)

Platform-specific GPU auto-detection with graceful CPU fallback:

**macOS**:
```
Metal available? → Use Metal GPU
                 ↓
                 Use CPU
```

**Linux/Windows**:
```
CUDA available? → Use CUDA GPU
                ↓
                Use CPU
```

**Implementation** (`provider.rs:99-124`):
```rust
"auto" => {
    #[cfg(target_os = "macos")]
    {
        if let Ok(metal) = Device::new_metal(0) {
            info!("Auto-detected Metal device");
            metal
        } else {
            info!("Auto-detected CPU device (Metal unavailable)");
            Device::Cpu
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        match Device::cuda_if_available(0) {
            Ok(Device::Cuda(d)) => {
                info!("Auto-detected CUDA device");
                Device::Cuda(d)
            }
            _ => {
                info!("Auto-detected CPU device (CUDA unavailable)");
                Device::Cpu
            }
        }
    }
}
```

#### 2. CPU Mode (`"cpu"`)

Forces CPU execution on all platforms. Always available.

**Use Cases**:
- Consistent performance testing
- Systems without GPU
- Debugging
- Low-power operation

#### 3. CUDA Mode (`"cuda"`)

Explicit CUDA device selection:

**Linux/Windows**: Attempts CUDA, returns error if unavailable
**macOS**: Logs warning, falls back to CPU (CUDA not supported)

**Implementation** (`provider.rs:53-76`):
```rust
"cuda" => {
    #[cfg(target_os = "macos")]
    {
        warn!("CUDA requested but not available on macOS, using CPU");
        info!("Hint: Use device='metal' for GPU acceleration");
        Device::Cpu
    }

    #[cfg(not(target_os = "macos"))]
    {
        match Device::cuda_if_available(0) {
            Ok(Device::Cuda(d)) => Device::Cuda(d),
            Err(e) => return Err(anyhow!("CUDA not available: {}", e)),
        }
    }
}
```

#### 4. Metal Mode (`"metal"`)

Explicit Metal device selection:

**macOS**: Attempts Metal, returns error if unavailable
**Linux/Windows**: Logs warning, falls back to CPU (Metal not supported)

**Implementation** (`provider.rs:78-93`):
```rust
"metal" => {
    #[cfg(not(target_os = "macos"))]
    {
        warn!("Metal requested but only available on macOS, using CPU");
        Device::Cpu
    }

    #[cfg(target_os = "macos")]
    {
        Device::new_metal(0).map_err(|e| {
            anyhow!("Metal not available: {}", e)
        })?
    }
}
```

## Cross-Platform Compilation

### Metal Feature Gating

Metal GPU support is **compile-time gated** to macOS only:

**Workspace `Cargo.toml`**:
```toml
# No Metal feature at workspace level (was causing objc_exception on Linux)
candle-core = "0.9"
```

**`llmspell-providers/Cargo.toml`**:
```toml
# Platform-specific dependencies for GPU support
[target.'cfg(target_os = "macos")'.dependencies]
candle-core = { workspace = true, features = ["metal"] }

[target.'cfg(not(target_os = "macos"))'.dependencies]
candle-core = { workspace = true }
```

This ensures:
- ✅ Metal symbols only compiled on macOS
- ✅ No Objective-C dependencies on Linux
- ✅ Zero compilation errors across platforms
- ✅ Optimal binary size (no unused GPU code)

### Validation Tests

GPU detection logic is validated via cross-platform tests:

**Test File**: `llmspell-providers/tests/gpu_detection_test.rs`

**Linux Test Results** (validated 2025-11-01):
```
✅ CPU device initialization: OK
✅ Auto device detection: OK (no panic)
✅ CUDA device: Not available (expected if no CUDA)
✅ Invalid device: Fallback to CPU
✅ Metal on Linux: Fallback to CPU (expected behavior)

test result: ok. 5 passed; 0 failed; 0 ignored
```

## Performance Characteristics

### GPU Acceleration Support by Model Type

| Model Type | Architecture | Metal (macOS) | CUDA (Linux) | CPU (Universal) |
|------------|--------------|---------------|--------------|-----------------|
| GGUF (LLaMA) | Decoder-only | ⚠️ Blocked by RMS-norm | ✅ Supported | ✅ Supported |
| T5 (Flan-T5) | Encoder-decoder | ✅ Working | ✅ Supported | ✅ Supported |

**Note**: GGUF models on Metal currently encounter RMS-normalization issues in candle-core. T5 models work correctly with Metal GPU acceleration.

### Recommended Models by Platform

**macOS (Apple Silicon - Metal GPU)**:
```bash
# Best for Metal GPU acceleration
llmspell model pull flan-t5-small@candle   # 80M params, fast
llmspell model pull flan-t5-base@candle    # 250M params, balanced
llmspell model pull flan-t5-large@candle   # 780M params, powerful
```

**Linux/Windows (CUDA GPU)**:
```bash
# GGUF models work well with CUDA
llmspell model pull tinyllama@candle       # 1.1B params, quantized
llmspell model pull phi-2@candle           # 2.7B params, quantized
llmspell model pull qwen2-0.5b@candle      # 0.5B params, quantized

# T5 models also work
llmspell model pull flan-t5-small@candle
```

**All Platforms (CPU)**:
```bash
# Lighter models recommended for CPU
llmspell model pull flan-t5-small@candle   # Fast CPU inference
llmspell model pull qwen2-0.5b@candle      # Quantized for efficiency
```

## Runtime Behavior

### Initialization Logs

**macOS with Metal GPU**:
```
INFO Initializing Candle provider: device=auto
INFO Auto-detected Metal device for Candle (Apple Silicon)
INFO Candle provider initialized with device: Metal(Metal(0))
```

**Linux without CUDA**:
```
INFO Initializing Candle provider: device=auto
INFO Auto-detected CPU device for Candle (CUDA unavailable)
INFO Candle provider initialized with device: Cpu
```

**Linux with CUDA**:
```
INFO Initializing Candle provider: device=auto
INFO Auto-detected CUDA device for Candle
INFO Candle provider initialized with device: Cuda(CudaDevice(0))
```

### Error Handling

All GPU initialization errors are **graceful** with helpful messages:

**CUDA requested on macOS**:
```
WARN CUDA requested but not available on macOS, using CPU
INFO Hint: Use device='metal' for GPU acceleration on Apple Silicon
```

**Metal requested on Linux**:
```
WARN Metal requested but only available on macOS, using CPU
```

**CUDA explicitly requested but unavailable**:
```
ERROR CUDA device requested but not available: CUDA driver not found
Error: CUDA not available: CUDA driver not found
```

## Configuration Examples

### Lua Script Configuration

```lua
-- Auto-detect best GPU (recommended)
local candle = require("candle")
local provider = candle.new({
    model = "flan-t5-small",
    device = "auto"
})

-- Force CPU (consistent performance)
local provider_cpu = candle.new({
    model = "tinyllama",
    device = "cpu"
})

-- Explicit Metal (macOS only, errors on Linux)
local provider_metal = candle.new({
    model = "flan-t5-base",
    device = "metal"
})

-- Explicit CUDA (Linux/Windows, fallback on macOS)
local provider_cuda = candle.new({
    model = "phi-2",
    device = "cuda"
})
```

### Rust API Usage

```rust
use llmspell_providers::local::candle::CandleProvider;

// Auto-detect (recommended)
let provider = CandleProvider::new(
    "flan-t5-small".to_string(),
    None,
    "auto".to_string()
)?;

// Platform-specific optimal
#[cfg(target_os = "macos")]
let provider = CandleProvider::new(
    "flan-t5-small".to_string(),
    None,
    "metal".to_string()
)?;

#[cfg(not(target_os = "macos"))]
let provider = CandleProvider::new(
    "tinyllama".to_string(),
    None,
    "cuda".to_string()  // Falls back to CPU if CUDA unavailable
)?;
```

## Troubleshooting

### Metal Not Available on macOS

**Symptom**: `Metal not available` error on Apple Silicon Mac

**Causes**:
1. Intel Mac (Metal not supported for inference)
2. macOS version too old (requires macOS 11+)
3. Metal framework not available

**Solution**:
```bash
# Use CPU mode explicitly
device="cpu"

# Or let auto-detection fallback
device="auto"  # Will use CPU if Metal unavailable
```

### CUDA Not Available on Linux

**Symptom**: `CUDA not available` error

**Causes**:
1. NVIDIA GPU not present
2. CUDA drivers not installed
3. CUDA version incompatible

**Solution**:
```bash
# Install CUDA drivers (Ubuntu/Debian)
sudo apt install nvidia-cuda-toolkit

# Verify CUDA
nvidia-smi

# Or use CPU mode
device="cpu"
```

### Compilation Errors on Linux

**Symptom**: `cannot execute 'cc1obj'` or `objc_exception` errors

**Cause**: Outdated repository (fixed in Phase 13b.1.1)

**Solution**:
```bash
# Pull latest changes
git pull origin Phase-13b

# Verify Metal feature is platform-gated
grep -A 2 "cfg(target_os" llmspell-providers/Cargo.toml
# Should show:
# [target.'cfg(target_os = "macos")'.dependencies]
# candle-core = { workspace = true, features = ["metal"] }
```

## Implementation References

**Primary Implementation**: `llmspell-providers/src/local/candle/provider.rs`

| Feature | Lines | Description |
|---------|-------|-------------|
| Device Selection | 52-130 | Main GPU detection logic |
| CUDA Mode | 53-76 | CUDA with platform gating |
| Metal Mode | 78-93 | Metal with platform gating |
| Auto Mode | 99-124 | Platform-aware auto-detection |
| CPU Mode | 95-98 | Universal CPU fallback |

**Validation Tests**: `llmspell-providers/tests/gpu_detection_test.rs`
- ✅ 5 tests covering all device modes
- ✅ Platform-specific behavior validation
- ✅ Graceful error handling verification

## Related Documentation

- **Phase 13b Plan**: `/docs/in-progress/phase-13b-plan.md`
- **TODO Task 13b.1.1**: Linux compilation fixes
- **TODO Task 13b.1.2**: GPU detection validation (this document)
- **Candle Documentation**: https://github.com/huggingface/candle

## Future Enhancements

- [ ] DirectML support for Windows (alternative to CUDA)
- [ ] Vulkan backend (cross-platform GPU)
- [ ] ROCm support for AMD GPUs on Linux
- [ ] GPU memory management and multi-GPU support
- [ ] Auto-detection of optimal device based on model size
