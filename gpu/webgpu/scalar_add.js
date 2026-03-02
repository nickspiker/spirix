// WebGPU ScalarF4E4 Addition - JavaScript Host Code
// Matches HIP host API but uses WebGPU

class ScalarF4E4GPU {
    constructor() {
        this.device = null;
        this.pipeline = null;
        this.bindGroupLayout = null;
    }

    // Initialize WebGPU device and compile shader
    async init() {
        // Check WebGPU support
        if (!navigator.gpu) {
            throw new Error("WebGPU not supported! Use Chrome/Edge 113+, Firefox Nightly, or Safari 17+");
        }

        // Request adapter and device
        const adapter = await navigator.gpu.requestAdapter();
        if (!adapter) {
            throw new Error("No GPU adapter found");
        }

        this.device = await adapter.requestDevice();

        // Load shader code
        const shaderResponse = await fetch('scalar_add.wgsl');
        const shaderCode = await shaderResponse.text();

        // Create shader module
        const shaderModule = this.device.createShaderModule({
            code: shaderCode
        });

        // Create bind group layout
        this.bindGroupLayout = this.device.createBindGroupLayout({
            entries: [
                { binding: 0, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'read-only-storage' } },
                { binding: 1, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'read-only-storage' } },
                { binding: 2, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'storage' } }
            ]
        });

        // Create pipeline
        this.pipeline = this.device.createComputePipeline({
            layout: this.device.createPipelineLayout({
                bindGroupLayouts: [this.bindGroupLayout]
            }),
            compute: {
                module: shaderModule,
                entryPoint: 'scalar_add'
            }
        });

        console.log("✓ WebGPU initialized");
        console.log(`  Device: ${adapter.name || 'Unknown GPU'}`);
    }

    // Pack ScalarF4E4 (fraction, exponent) into u32
    packScalar(fraction, exponent) {
        return ((exponent & 0xFFFF) << 16) | (fraction & 0xFFFF);
    }

    // Unpack u32 into ScalarF4E4 with sign extension
    unpackScalar(packed) {
        let frac = packed & 0xFFFF;
        let exp = (packed >> 16) & 0xFFFF;

        // Sign-extend from 16 to 32 bits
        if (frac & 0x8000) frac |= 0xFFFF0000;
        if (exp & 0x8000) exp |= 0xFFFF0000;

        return { fraction: frac >> 0, exponent: exp >> 0 };  // Convert to signed i32
    }

    // Add two arrays of ScalarF4E4 values on GPU
    async add(a_array, b_array) {
        const n = a_array.length;

        // Pack input arrays (fraction, exponent) → u32
        const a_packed = new Uint32Array(n);
        const b_packed = new Uint32Array(n);

        for (let i = 0; i < n; i++) {
            a_packed[i] = this.packScalar(a_array[i].fraction, a_array[i].exponent);
            b_packed[i] = this.packScalar(b_array[i].fraction, b_array[i].exponent);
        }

        // Create GPU buffers
        const bufferSize = n * 4; // 4 bytes per u32

        const a_buffer = this.device.createBuffer({
            size: bufferSize,
            usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
            mappedAtCreation: true
        });
        new Uint32Array(a_buffer.getMappedRange()).set(a_packed);
        a_buffer.unmap();

        const b_buffer = this.device.createBuffer({
            size: bufferSize,
            usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
            mappedAtCreation: true
        });
        new Uint32Array(b_buffer.getMappedRange()).set(b_packed);
        b_buffer.unmap();

        const result_buffer = this.device.createBuffer({
            size: bufferSize,
            usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
        });

        const staging_buffer = this.device.createBuffer({
            size: bufferSize,
            usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST
        });

        // Create bind group
        const bindGroup = this.device.createBindGroup({
            layout: this.bindGroupLayout,
            entries: [
                { binding: 0, resource: { buffer: a_buffer } },
                { binding: 1, resource: { buffer: b_buffer } },
                { binding: 2, resource: { buffer: result_buffer } }
            ]
        });

        // Dispatch compute shader
        const workgroupSize = 256;
        const workgroups = Math.ceil(n / workgroupSize);

        const commandEncoder = this.device.createCommandEncoder();
        const passEncoder = commandEncoder.beginComputePass();
        passEncoder.setPipeline(this.pipeline);
        passEncoder.setBindGroup(0, bindGroup);
        passEncoder.dispatchWorkgroups(workgroups);
        passEncoder.end();

        // Copy result to staging buffer
        commandEncoder.copyBufferToBuffer(result_buffer, 0, staging_buffer, 0, bufferSize);

        // Submit and wait
        this.device.queue.submit([commandEncoder.finish()]);
        await staging_buffer.mapAsync(GPUMapMode.READ);

        // Read results
        const result_packed = new Uint32Array(staging_buffer.getMappedRange()).slice();
        staging_buffer.unmap();

        // Unpack results
        const results = [];
        for (let i = 0; i < n; i++) {
            results.push(this.unpackScalar(result_packed[i]));
        }

        // Cleanup
        a_buffer.destroy();
        b_buffer.destroy();
        result_buffer.destroy();
        staging_buffer.destroy();

        return results;
    }

    // Benchmark (like our HIP benchmarks)
    async benchmark(n, iterations) {
        // Generate test data
        const a_array = [];
        const b_array = [];

        for (let i = 0; i < n; i++) {
            a_array.push({ fraction: 20000 + (i % 100), exponent: (i % 20) - 10 });
            b_array.push({ fraction: 15000 + (i % 100), exponent: (i % 10) - 5 });
        }

        // Warmup
        await this.add(a_array, b_array);

        // Benchmark
        const start = performance.now();
        for (let i = 0; i < iterations; i++) {
            await this.add(a_array, b_array);
        }
        const end = performance.now();

        const elapsed_ms = end - start;
        const total_ops = n * iterations;
        const ops_per_sec = total_ops / (elapsed_ms / 1000);
        const gops = ops_per_sec / 1e9;

        return {
            time_ms: elapsed_ms,
            gops: gops,
            ops_per_sec: ops_per_sec
        };
    }
}

// Export for use in HTML
if (typeof module !== 'undefined' && module.exports) {
    module.exports = ScalarF4E4GPU;
}
