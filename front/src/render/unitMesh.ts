export function createUnitMesh(device: GPUDevice) {
  const vertices = new Float32Array([
    -0.5, 0, 0,
     0.5, 0, 0,
     0.0, 1, 0,
  ]);

  const buffer = device.createBuffer({
    size: vertices.byteLength,
    usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
  });

  device.queue.writeBuffer(buffer, 0, vertices);
  return buffer;
}
