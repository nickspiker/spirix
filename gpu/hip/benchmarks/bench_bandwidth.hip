#include <hip/hip_runtime.h>
#include <stdio.h>

int main() {
    hipDeviceProp_t prop;
    hipGetDeviceProperties(&prop, 0);
    
    printf("=== GPU Memory Analysis ===\n");
    printf("Device: %s\n", prop.name);
    printf("Memory Clock Rate: %.2f GHz\n", prop.memoryClockRate / 1e6);
    printf("Memory Bus Width: %d bits\n", prop.memoryBusWidth);
    printf("Peak Memory Bandwidth: %.2f GB/s\n", 
           2.0 * prop.memoryClockRate * (prop.memoryBusWidth / 8) / 1e6);
    printf("Compute Capability: %d.%d\n", prop.major, prop.minor);
    printf("Max Threads per Block: %d\n", prop.maxThreadsPerBlock);
    printf("Multiprocessor Count: %d\n", prop.multiProcessorCount);
    
    printf("\n=== Theoretical Analysis ===\n");
    double bandwidth_gbs = 2.0 * prop.memoryClockRate * (prop.memoryBusWidth / 8) / 1e6;
    
    // ScalarF4E4: 4 bytes (2x int16)
    // Operation: Load 2 inputs (8 bytes) + Store 1 output (4 bytes) = 12 bytes
    double max_ops_bandwidth = bandwidth_gbs / 12.0;  // In billions
    
    printf("Bytes per operation: 12 (2 inputs + 1 output)\n");
    printf("Max OPS from bandwidth: %.2f GOPS\n", max_ops_bandwidth);
    printf("Observed Add/Sub/Mul: ~10.5 GOPS\n");
    printf("Bandwidth utilization: %.1f%%\n", (10.5 / max_ops_bandwidth) * 100);
    
    return 0;
}
