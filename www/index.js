import * as wasm from "quadratic-map-attractor";

async function init() {
    console.log("Initializing WASM Core...");
    // 1 million particles, base_scale 2.0
    const config = new wasm.AttractorConfig(1000000, 2.0);
    const numParticles = config.num_particles();
    console.log(`Initialized ${numParticles} particles in Rust.`);

    console.log("Initializing WebGPU...");
    const canvas = document.getElementById("canvas");
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    if (!navigator.gpu) {
        alert("WebGPU not supported on this browser.");
        throw new Error("WebGPU not supported.");
    }

    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) {
        alert("No appropriate WebGPU adapter found.");
        throw new Error("No WebGPU adapter found.");
    }

    const device = await adapter.requestDevice();
    const context = canvas.getContext("webgpu");

    const canvasFormat = navigator.gpu.getPreferredCanvasFormat();
    context.configure({
        device: device,
        format: canvasFormat,
        alphaMode: "premultiplied",
    });

    console.log("WebGPU Initialized Successfully.");

    // Update UI Events
    const kSlider = document.getElementById("k_slider");
    const kVal = document.getElementById("k_val");
    let kValue = parseFloat(kSlider.value);
    let escapeRadius = 5.0; // Phase 4 specifies 5.0 as default
    kSlider.addEventListener("input", (e) => {
        kValue = parseFloat(e.target.value);
        kVal.innerText = kValue.toFixed(2);
    });

    // Handle Window Resize
    window.addEventListener("resize", () => {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
    });

    // --- Phase 2 / Phase 3 Data Pipeline & Buffers ---

    // Create Float32Array views from WASM memory
    const statesArray = new Float32Array(wasm.memory.buffer, config.states_ptr(), numParticles * 8);
    const constantsArray = new Float32Array(wasm.memory.buffer, config.constants_ptr(), numParticles * 8);

    // Create buffers
    const stateBuffer = device.createBuffer({
        size: statesArray.byteLength,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    device.queue.writeBuffer(stateBuffer, 0, statesArray);

    const constantsBuffer = device.createBuffer({
        size: constantsArray.byteLength,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
    });
    device.queue.writeBuffer(constantsBuffer, 0, constantsArray);

    // Uniform buffer (vec4<f32>: k, escape_radius, padding, padding)
    const uniformBuffer = device.createBuffer({
        size: 16,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    console.log("Memory mapped and transferred to GPU.");

    // Load shaders
    const computeShaderCode = await fetch('./shaders/compute.wgsl').then(res => res.text());
    const computeModule = device.createShaderModule({ code: computeShaderCode });

    const renderShaderCode = await fetch('./shaders/render.wgsl').then(res => res.text());
    const renderModule = device.createShaderModule({ code: renderShaderCode });

    // Compute pipeline
    const computePipeline = device.createComputePipeline({
        layout: "auto",
        compute: {
            module: computeModule,
            entryPoint: "main"
        }
    });

    const computeBindGroup = device.createBindGroup({
        layout: computePipeline.getBindGroupLayout(0),
        entries: [
            { binding: 0, resource: { buffer: stateBuffer } },
            { binding: 1, resource: { buffer: constantsBuffer } },
            { binding: 2, resource: { buffer: uniformBuffer } }
        ]
    });

    // Render pipeline
    const renderPipeline = device.createRenderPipeline({
        layout: "auto",
        vertex: {
            module: renderModule,
            entryPoint: "vs_main"
        },
        fragment: {
            module: renderModule,
            entryPoint: "fs_main",
            targets: [{
                format: canvasFormat,
                blend: {
                    color: { srcFactor: "src-alpha", dstFactor: "one", operation: "add" },
                    alpha: { srcFactor: "zero", dstFactor: "one", operation: "add" }
                }
            }]
        },
        primitive: {
            topology: "point-list"
        }
    });

    const renderBindGroup = device.createBindGroup({
        layout: renderPipeline.getBindGroupLayout(0),
        entries: [
            { binding: 0, resource: { buffer: stateBuffer } }
        ]
    });

    // --- Main Loop (Phase 3) ---
    const workgroupCount = Math.ceil(numParticles / 256);

    function frame() {
        // Update uniform before compute
        const uniformData = new Float32Array([kValue, escapeRadius, 0.0, 0.0]);
        device.queue.writeBuffer(uniformBuffer, 0, uniformData);

        const commandEncoder = device.createCommandEncoder();

        // 1. Compute Pass
        const computePass = commandEncoder.beginComputePass();
        computePass.setPipeline(computePipeline);
        computePass.setBindGroup(0, computeBindGroup);
        computePass.dispatchWorkgroups(workgroupCount);
        computePass.end();

        // 2. Render Pass
        const renderPass = commandEncoder.beginRenderPass({
            colorAttachments: [{
                view: context.getCurrentTexture().createView(),
                clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
                loadOp: "clear",
                storeOp: "store"
            }]
        });

        renderPass.setPipeline(renderPipeline);
        renderPass.setBindGroup(0, renderBindGroup);
        renderPass.draw(numParticles, 1, 0, 0);
        renderPass.end();

        device.queue.submit([commandEncoder.finish()]);

        requestAnimationFrame(frame);
    }

    // Start rendering loop
    requestAnimationFrame(frame);
}

init().catch(console.error);
