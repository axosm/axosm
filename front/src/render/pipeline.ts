import unitShader from "../shader/units.wgsl?raw";

export function createPipeline(
  device: GPUDevice,
  format: GPUTextureFormat
) {
  return device.createRenderPipeline({
    layout: "auto",
    vertex: {
      module: device.createShaderModule({ code: unitShader }),
      entryPoint: "vs_main",
      buffers: [
        {
          arrayStride: 12,
          attributes: [{ shaderLocation: 0, offset: 0, format: "float32x3" }],
        },
        {
          arrayStride: 16,
          stepMode: "instance",
          attributes: [
            { shaderLocation: 1, offset: 0, format: "float32x3" },
            { shaderLocation: 2, offset: 12, format: "float32" },
          ],
        },
      ],
    },
    fragment: {
      module: device.createShaderModule({ code: unitShader }),
      entryPoint: "fs_main",
      targets: [{ format }],
    },
    primitive: {
      topology: "triangle-list",
    },
  });
}
