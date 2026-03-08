#!/usr/bin/env python3
"""
Minimal CUDA check: run after `cudamgr use <ver>` and adding PATH.
Verifies PyTorch sees CUDA and prints runtime version (should match active cudamgr version).
"""
import sys

def main():
    try:
        import torch
    except ImportError:
        print("PyTorch not installed. Run: pip install torch")
        sys.exit(1)

    print("PyTorch version:", torch.__version__)
    print("CUDA available:", torch.cuda.is_available())
    if torch.cuda.is_available():
        print("CUDA version (runtime):", torch.version.cuda)
        print("Device count:", torch.cuda.device_count())
        for i in range(torch.cuda.device_count()):
            print(f"  Device {i}:", torch.cuda.get_device_name(i))
        # Quick tensor test
        x = torch.randn(3, 3, device="cuda")
        print("Sample tensor on GPU:", x.device)
        print("OK — PyTorch is using CUDA.")
    else:
        print("CUDA not available. Check:")
        print("  1. You ran: cudamgr use <version>")
        print("  2. You added the printed PATH in this terminal (or permanently)")
        print("  3. nvcc --version shows the expected version")
        sys.exit(1)

if __name__ == "__main__":
    main()
