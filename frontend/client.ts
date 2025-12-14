// client.ts
// run in browser; expects Rust server at http://127.0.0.1:3000

type Unit = { id: number, player_id: number, x: number, y: number };

const canvas = document.getElementById("gpuCanvas") as HTMLCanvasElement;
const playerIdInput = document.getElementById("playerId") as HTMLInputElement;
const connectBtn = document.getElementById("connect") as HTMLButtonElement;
const logDiv = document.getElementById("log") as HTMLDivElement;

const GRID_SIZE = 20;
const CELL_PIX = Math.floor(canvas.width / GRID_SIZE);

let units: Unit[] = [];
let device: GPUDevice;
let context: GPUCanvasContext;

async function initWebGPU() {
  if (!navigator.gpu) {
    alert("WebGPU not supported in this browser.");
    throw new Error("no webgpu");
  }
  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) throw new Error("no adapter");
  device = await adapter.requestDevice();
  context = canvas.getContext("webgpu") as unknown as GPUCanvasContext;
  const format = navigator.gpu.getPreferredCanvasFormat();
  context.configure({ device, format, alphaMode: "opaque" });
}

function log(msg: string) {
  const d = document.createElement("div");
  d.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
  logDiv.prepend(d);
  if (logDiv.childElementCount > 20) logDiv.removeChild(logDiv.lastChild!);
}

// We'll render grid + units using 2D rectangles transformed into clip space via a simple vertex shader.
// Prepare a very small pipeline that draws colored rectangles by vertex positions.

const vertexShaderWGSL = `
struct VertexOut {
  @builtin(position) pos: vec4<f32>;
  @location(0) frag_uv: vec2<f32>;
  @location(1) color: vec3<f32>;
};

@vertex
fn vs(@location(0) position: vec2<f32>, @location(1) color: vec3<f32>) -> VertexOut {
  var out: VertexOut;
  out.pos = vec4<f32>(position, 0.0, 1.0);
  out.frag_uv = position;
  out.color = color;
  return out;
}
`;

const fragmentShaderWGSL = `
@fragment
fn fs(@location(1) color: vec3<f32>) -> @location(0) vec4<f32> {
  return vec4<f32>(color, 1.0);
}
`;

let pipeline: GPURenderPipeline;
let vertexBuffer: GPUBuffer;
let verticesCount = 0;

function makeQuadVertices(x0: number, y0: number, x1: number, y1: number, color: [number,number,number]) {
  // positions are in clip space -1..1 ; convert pixel coords
  // x0,y0 top-left pixel coords; x1,y1 bottom-right
  const nx0 = (x0 / canvas.width) * 2 - 1;
  const ny0 = -((y0 / canvas.height) * 2 - 1);
  const nx1 = (x1 / canvas.width) * 2 - 1;
  const ny1 = -((y1 / canvas.height) * 2 - 1);

  // two triangles (6 vertices), each vertex has position vec2 + color vec3
  return new Float32Array([
    nx0, ny0, color[0], color[1], color[2],
    nx1, ny0, color[0], color[1], color[2],
    nx1, ny1, color[0], color[1], color[2],

    nx0, ny0, color[0], color[1], color[2],
    nx1, ny1, color[0], color[1], color[2],
    nx0, ny1, color[0], color[1], color[2],
  ]);
}

async function preparePipeline() {
  const format = navigator.gpu.getPreferredCanvasFormat();
  const moduleVS = device.createShaderModule({ code: vertexShaderWGSL });
  const moduleFS = device.createShaderModule({ code: fragmentShaderWGSL });

  pipeline = device.createRenderPipeline({
    layout: "auto",
    vertex: {
      module: moduleVS,
      entryPoint: "vs",
      buffers: [
        {
          arrayStride: 5 * 4,
          attributes: [
            { shaderLocation: 0, offset: 0, format: "float32x2" }, // pos
            { shaderLocation: 1, offset: 2 * 4, format: "float32x3" }, // color
          ],
        },
      ],
    },
    fragment: {
      module: moduleFS,
      entryPoint: "fs",
      targets: [{ format }],
    },
    primitive: {
      topology: "triangle-list",
    },
  });
}

// render grid + units
async function render() {
  // build vertex data for grid lines and units
  const parts: Float32Array[] = [];

  // grid cell outlines (thin rectangles)
  for (let gx = 0; gx < GRID_SIZE; gx++) {
    for (let gy = 0; gy < GRID_SIZE; gy++) {
      const x = gx * CELL_PIX;
      const y = gy * CELL_PIX;
      const thickness = 1;
      // background cell rect (very dark)
      parts.push(makeQuadVertices(x, y, x + CELL_PIX - 1, y + CELL_PIX - 1, [0.06, 0.06, 0.08]));
    }
  }

  // draw units
  for (const u of units) {
    const x = u.x * CELL_PIX;
    const y = u.y * CELL_PIX;
    const color = u.player_id === Number(playerIdInput.value) ? [0.2, 0.9, 0.2] : [0.9, 0.2, 0.2];
    parts.push(makeQuadVertices(x + 4, y + 4, x + CELL_PIX - 4, y + CELL_PIX - 4, color));
  }

  const totalLen = parts.reduce((s, p) => s + p.length, 0);
  const merged = new Float32Array(totalLen);
  let offset = 0;
  for (const p of parts) {
    merged.set(p, offset);
    offset += p.length;
  }
  verticesCount = merged.length / 5;

  // create GPU buffer
  if (vertexBuffer) vertexBuffer.destroy();
  vertexBuffer = device.createBuffer({
    size: merged.byteLength,
    usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    mappedAtCreation: true,
  });
  new Float32Array(vertexBuffer.getMappedRange()).set(merged);
  vertexBuffer.unmap();

  // command
  const commandEncoder = device.createCommandEncoder();
  const textureView = context.getCurrentTexture().createView();
  const pass = commandEncoder.beginRenderPass({
    colorAttachments: [{
      view: textureView,
      clearValue: { r: 0, g: 0, b: 0, a: 1 },
      loadOp: "clear",
      storeOp: "store",
    }]
  });

  pass.setPipeline(pipeline);
  pass.setVertexBuffer(0, vertexBuffer);
  pass.draw(verticesCount, 1, 0, 0);
  pass.end();

  device.queue.submit([commandEncoder.finish()]);
}

// Poll state every second
async function pollLoop() {
  const pid = Number(playerIdInput.value);
  try {
    const res = await fetch(`http://127.0.0.1:3000/api/state?player_id=${pid}`);
    if (res.ok) {
      const data = await res.json();
      units = data.units;
      await render();
    }
  } catch (e) {
    // ignore
  }
  setTimeout(pollLoop, 1000);
}

// click to send move
canvas.addEventListener("click", async (ev) => {
  const rect = canvas.getBoundingClientRect();
  const cx = ev.clientX - rect.left;
  const cy = ev.clientY - rect.top;
  const gx = Math.floor(cx / CELL_PIX);
  const gy = Math.floor(cy / CELL_PIX);

  // find the player's unit id (prototype: each player has one unit)
  const myPid = Number(playerIdInput.value);
  const mine = units.find(u => u.player_id === myPid);
  if (!mine) {
    log("no unit found for player");
    return;
  }

  // send move order
  const payload = { player_id: myPid, unit_id: mine.id, to_x: gx, to_y: gy };
  try {
    const r = await fetch("http://127.0.0.1:3000/api/move", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    const j = await r.json();
    log(`move requested to (${gx},${gy}) arrival=${j.arrival_time}`);
  } catch (e) {
    log("move failed");
  }
});

// SSE connect
let es: EventSource | null = null;
connectBtn.addEventListener("click", () => {
  if (es) {
    es.close();
    es = null;
    connectBtn.textContent = "Connect SSE";
    log("SSE disconnected");
    return;
  }
  const pid = Number(playerIdInput.value);
  es = new EventSource(`http://127.0.0.1:3000/api/events?player_id=${pid}`);
  es.onopen = () => {
    connectBtn.textContent = "Disconnect SSE";
    log("SSE connected");
  };
  es.onmessage = (evt) => {
    try {
      const d = JSON.parse(evt.data);
      if (d.type === "encounter" || d.r#type === "encounter" || d["type"] === "encounter") {
        log(`Encounter between ${d.player_a ?? d.playerA ?? d.player_a} and ${d.player_b ?? d.playerB ?? d.player_b} at (${d.x},${d.y})`);
        // optionally highlight the cell visually (left as exercise)
      } else {
        log(`event: ${evt.data}`);
      }
    } catch (e) {
      log(`event: ${evt.data}`);
    }
  };
  es.onerror = (e) => {
    log("SSE error");
  };
});

// initialize everything
(async () => {
  await initWebGPU();
  await preparePipeline();
  await pollLoop();
})();
