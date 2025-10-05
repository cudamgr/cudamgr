use cudamgr::system::gpu::{DefaultGpuDetector, GpuDetector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("CudaMgr GPU Detection Example");
    println!("=============================");

    let detector = DefaultGpuDetector::new();

    // Detect all GPUs
    println!("\nDetecting all GPUs...");
    match detector.detect_gpus().await {
        Ok(gpus) => {
            if gpus.is_empty() {
                println!("No GPUs detected.");
            } else {
                println!("Found {} GPU(s):", gpus.len());
                for (i, gpu) in gpus.iter().enumerate() {
                    println!("  GPU {}: {}", i + 1, gpu.name);
                    println!("    Vendor: {:?}", gpu.vendor);
                    if let Some(memory) = gpu.memory_mb {
                        println!("    Memory: {} MB", memory);
                    }
                    if let Some(capability) = gpu.compute_capability {
                        println!("    Compute Capability: {}.{}", capability.0, capability.1);
                    }
                    if let Some(driver) = &gpu.driver_version {
                        println!("    Driver Version: {}", driver);
                    }
                    if let Some(pci_id) = &gpu.pci_id {
                        println!("    PCI ID: {}", pci_id);
                    }
                    println!("    CUDA Compatible: {}", gpu.is_cuda_compatible());
                    println!();
                }
            }
        }
        Err(e) => {
            println!("Error detecting GPUs: {}", e);
        }
    }

    // Detect only NVIDIA GPUs
    println!("Detecting NVIDIA GPUs specifically...");
    match detector.detect_nvidia_gpus().await {
        Ok(nvidia_gpus) => {
            if nvidia_gpus.is_empty() {
                println!("No NVIDIA GPUs detected.");
            } else {
                println!("Found {} NVIDIA GPU(s):", nvidia_gpus.len());
                for (i, gpu) in nvidia_gpus.iter().enumerate() {
                    println!("  NVIDIA GPU {}: {}", i + 1, gpu.name);
                    if let Some(capability) = gpu.compute_capability {
                        println!("    Compute Capability: {}.{}", capability.0, capability.1);
                        
                        // Test some common CUDA requirements
                        println!("    Supports CUDA 11.0+ (CC 3.5+): {}", gpu.supports_compute_capability((3, 5)));
                        println!("    Supports CUDA 12.0+ (CC 5.0+): {}", gpu.supports_compute_capability((5, 0)));
                        println!("    Supports Tensor Cores (CC 7.0+): {}", gpu.supports_compute_capability((7, 0)));
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            println!("Error detecting NVIDIA GPUs: {}", e);
        }
    }

    Ok(())
}