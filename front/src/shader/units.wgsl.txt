struct VSOut {
  @builtin(position) position : vec4<f32>,
};

@group(0) @binding(0)
var<uniform> viewProj : mat4x4<f32>;

@vertex
fn vs_main(
  @location(0) pos : vec3<f32>,
  @location(1) instancePos : vec3<f32>,
  @location(2) scale : f32
) -> VSOut {
  var out : VSOut;
  let world = instancePos + pos * scale;
  out.position = viewProj * vec4(world, 1.0);
  return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
  return vec4(1.0, 0.8, 0.2, 1.0);
}
