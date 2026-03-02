// WebGPU ScalarF4E4 Operations - Unified JavaScript API
// Matches HIP host API but uses WebGPU for all operations

class ScalarF4E4GPU {
    constructor() {
        this.device = null;
        this.pipelines = {};
        this.bindGroupLayouts = {};
        this.reciprocalLUT = null;
    }

    // Initialize WebGPU device and compile all shaders
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

        // Load all shaders
        const operations = ['add', 'multiply', 'divide', 'sqrt'];
        const shaderFiles = {
            'add': 'scalar_add.wgsl',
            'multiply': 'scalar_multiply.wgsl',
            'divide': 'scalar_divide.wgsl',
            'sqrt': 'scalar_sqrt.wgsl'
        };

        for (const op of operations) {
            const shaderResponse = await fetch(shaderFiles[op]);
            const shaderCode = await shaderResponse.text();

            const shaderModule = this.device.createShaderModule({
                code: shaderCode
            });

            // Create bind group layout (divide and sqrt have different layouts)
            if (op === 'sqrt') {
                // Sqrt: 1 input, 1 output
                this.bindGroupLayouts[op] = this.device.createBindGroupLayout({
                    entries: [
                        { binding: 0, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'read-only-storage' } },
                        { binding: 1, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'storage' } }
                    ]
                });
            } else if (op === 'divide') {
                // Divide: 2 inputs, 1 output, 1 LUT
                this.bindGroupLayouts[op] = this.device.createBindGroupLayout({
                    entries: [
                        { binding: 0, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'read-only-storage' } },
                        { binding: 1, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'read-only-storage' } },
                        { binding: 2, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'storage' } },
                        { binding: 3, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'read-only-storage' } }
                    ]
                });
            } else {
                // Add, multiply: 2 inputs, 1 output
                this.bindGroupLayouts[op] = this.device.createBindGroupLayout({
                    entries: [
                        { binding: 0, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'read-only-storage' } },
                        { binding: 1, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'read-only-storage' } },
                        { binding: 2, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'storage' } }
                    ]
                });
            }

            // Create pipeline
            this.pipelines[op] = this.device.createComputePipeline({
                layout: this.device.createPipelineLayout({
                    bindGroupLayouts: [this.bindGroupLayouts[op]]
                }),
                compute: {
                    module: shaderModule,
                    entryPoint: `scalar_${op}`
                }
            });
        }

        // Create reciprocal LUT for division
        this.reciprocalLUT = this.createReciprocalLUT();

        console.log("✓ WebGPU initialized");
        console.log(`  Device: ${adapter.name || 'Unknown GPU'}`);
        console.log(`  Operations: add, multiply, divide, sqrt`);
    }

    // Create reciprocal LUT (matches HIP constant memory)
    createReciprocalLUT() {
        const lut = new Int32Array(256);
        const lutValues = [
            0x4000, 0x3fc0, 0x3f80, 0x3f42, 0x3f03, 0x3ec6, 0x3e88, 0x3e4b,
            0x3e0f, 0x3dd3, 0x3d98, 0x3d5d, 0x3d22, 0x3ce8, 0x3cae, 0x3c75,
            0x3c3c, 0x3c03, 0x3bcb, 0x3b94, 0x3b5c, 0x3b25, 0x3aef, 0x3ab9,
            0x3a83, 0x3a4e, 0x3a19, 0x39e4, 0x39b0, 0x397c, 0x3949, 0x3916,
            0x38e3, 0x38b1, 0x387f, 0x384d, 0x381c, 0x37eb, 0x37ba, 0x3789,
            0x3759, 0x372a, 0x36fa, 0x36cb, 0x369d, 0x366e, 0x3640, 0x3612,
            0x35e5, 0x35b7, 0x358a, 0x355e, 0x3531, 0x3505, 0x34da, 0x34ae,
            0x3483, 0x3458, 0x342d, 0x3403, 0x33d9, 0x33af, 0x3385, 0x335c,
            0x3333, 0x330a, 0x32e1, 0x32b9, 0x3291, 0x3269, 0x3241, 0x321a,
            0x31f3, 0x31cc, 0x31a6, 0x317f, 0x3159, 0x3133, 0x310d, 0x30e8,
            0x30c3, 0x309e, 0x3079, 0x3054, 0x3030, 0x300c, 0x2fe8, 0x2fc4,
            0x2fa0, 0x2f7d, 0x2f5a, 0x2f37, 0x2f14, 0x2ef2, 0x2ecf, 0x2ead,
            0x2e8b, 0x2e69, 0x2e48, 0x2e26, 0x2e05, 0x2de4, 0x2dc3, 0x2da3,
            0x2d82, 0x2d62, 0x2d42, 0x2d22, 0x2d02, 0x2ce3, 0x2cc3, 0x2ca4,
            0x2c85, 0x2c66, 0x2c47, 0x2c29, 0x2c0b, 0x2bec, 0x2bce, 0x2bb0,
            0x2b93, 0x2b75, 0x2b58, 0x2b3a, 0x2b1d, 0x2b00, 0x2ae3, 0x2ac7,
            0x2aaa, 0x2a8e, 0x2a72, 0x2a55, 0x2a3a, 0x2a1e, 0x2a02, 0x29e7,
            0x29cb, 0x29b0, 0x2995, 0x297a, 0x295f, 0x2944, 0x292a, 0x2910,
            0x28f5, 0x28db, 0x28c1, 0x28a7, 0x288d, 0x2874, 0x285a, 0x2841,
            0x2828, 0x280f, 0x27f6, 0x27dd, 0x27c4, 0x27ab, 0x2793, 0x277a,
            0x2762, 0x274a, 0x2732, 0x271a, 0x2702, 0x26ea, 0x26d3, 0x26bb,
            0x26a4, 0x268c, 0x2675, 0x265e, 0x2647, 0x2630, 0x261a, 0x2603,
            0x25ed, 0x25d6, 0x25c0, 0x25aa, 0x2593, 0x257d, 0x2568, 0x2552,
            0x253c, 0x2526, 0x2511, 0x24fb, 0x24e6, 0x24d1, 0x24bc, 0x24a7,
            0x2492, 0x247d, 0x2468, 0x2454, 0x243f, 0x242a, 0x2416, 0x2402,
            0x23ee, 0x23d9, 0x23c5, 0x23b1, 0x239e, 0x238a, 0x2376, 0x2362,
            0x234f, 0x233c, 0x2328, 0x2315, 0x2302, 0x22ef, 0x22dc, 0x22c9,
            0x22b6, 0x22a3, 0x2290, 0x227e, 0x226b, 0x2259, 0x2246, 0x2234,
            0x2222, 0x220f, 0x21fd, 0x21eb, 0x21d9, 0x21c8, 0x21b6, 0x21a4,
            0x2192, 0x2181, 0x216f, 0x215e, 0x214d, 0x213b, 0x212a, 0x2119,
            0x2108, 0x20f7, 0x20e6, 0x20d5, 0x20c4, 0x20b3, 0x20a3, 0x2092,
            0x2082, 0x2071, 0x2061, 0x2050, 0x2040, 0x2030, 0x2020, 0x2010
        ];
        lut.set(lutValues);
        return lut;
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

    // Generic binary operation dispatcher
    async binaryOp(operation, a_array, b_array) {
        const n = a_array.length;

        // Pack input arrays
        const a_packed = new Uint32Array(n);
        const b_packed = new Uint32Array(n);

        for (let i = 0; i < n; i++) {
            a_packed[i] = this.packScalar(a_array[i].fraction, a_array[i].exponent);
            b_packed[i] = this.packScalar(b_array[i].fraction, b_array[i].exponent);
        }

        // Create GPU buffers
        const bufferSize = n * 4;

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

        // Create bind group (add LUT for division)
        const entries = [
            { binding: 0, resource: { buffer: a_buffer } },
            { binding: 1, resource: { buffer: b_buffer } },
            { binding: 2, resource: { buffer: result_buffer } }
        ];

        if (operation === 'divide') {
            // Create LUT buffer
            const lut_buffer = this.device.createBuffer({
                size: 256 * 4,
                usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
                mappedAtCreation: true
            });
            new Int32Array(lut_buffer.getMappedRange()).set(this.reciprocalLUT);
            lut_buffer.unmap();

            entries.push({ binding: 3, resource: { buffer: lut_buffer } });
        }

        const bindGroup = this.device.createBindGroup({
            layout: this.bindGroupLayouts[operation],
            entries: entries
        });

        // Dispatch compute shader
        const workgroupSize = 256;
        const workgroups = Math.ceil(n / workgroupSize);

        const commandEncoder = this.device.createCommandEncoder();
        const passEncoder = commandEncoder.beginComputePass();
        passEncoder.setPipeline(this.pipelines[operation]);
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

    // Unary operation (sqrt)
    async unaryOp(operation, a_array) {
        const n = a_array.length;

        // Pack input array
        const a_packed = new Uint32Array(n);
        for (let i = 0; i < n; i++) {
            a_packed[i] = this.packScalar(a_array[i].fraction, a_array[i].exponent);
        }

        // Create GPU buffers
        const bufferSize = n * 4;

        const a_buffer = this.device.createBuffer({
            size: bufferSize,
            usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
            mappedAtCreation: true
        });
        new Uint32Array(a_buffer.getMappedRange()).set(a_packed);
        a_buffer.unmap();

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
            layout: this.bindGroupLayouts[operation],
            entries: [
                { binding: 0, resource: { buffer: a_buffer } },
                { binding: 1, resource: { buffer: result_buffer } }
            ]
        });

        // Dispatch compute shader
        const workgroupSize = 256;
        const workgroups = Math.ceil(n / workgroupSize);

        const commandEncoder = this.device.createCommandEncoder();
        const passEncoder = commandEncoder.beginComputePass();
        passEncoder.setPipeline(this.pipelines[operation]);
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
        result_buffer.destroy();
        staging_buffer.destroy();

        return results;
    }

    // Public API methods
    async add(a_array, b_array) {
        return this.binaryOp('add', a_array, b_array);
    }

    async multiply(a_array, b_array) {
        return this.binaryOp('multiply', a_array, b_array);
    }

    async divide(a_array, b_array) {
        return this.binaryOp('divide', a_array, b_array);
    }

    async sqrt(a_array) {
        return this.unaryOp('sqrt', a_array);
    }

    // Benchmark (like our HIP benchmarks)
    async benchmark(operation, n, iterations) {
        // Generate test data
        const a_array = [];
        const b_array = [];

        for (let i = 0; i < n; i++) {
            a_array.push({ fraction: 20000 + (i % 100), exponent: (i % 20) - 10 });
            if (operation !== 'sqrt') {
                b_array.push({ fraction: 15000 + (i % 100), exponent: (i % 10) - 5 });
            }
        }

        // Warmup
        if (operation === 'sqrt') {
            await this[operation](a_array);
        } else {
            await this[operation](a_array, b_array);
        }

        // Benchmark
        const start = performance.now();
        for (let i = 0; i < iterations; i++) {
            if (operation === 'sqrt') {
                await this[operation](a_array);
            } else {
                await this[operation](a_array, b_array);
            }
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
