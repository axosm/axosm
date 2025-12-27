import { Vec3, add, scale, normalize } from "./vec3";

const t = (1 + Math.sqrt(5)) / 2;

export const ICOSAHEDRON_VERTICES: Vec3[] = [
  normalize([-1,  t,  0]),
  normalize([ 1,  t,  0]),
  normalize([-1, -t,  0]),
  normalize([ 1, -t,  0]),

  normalize([ 0, -1,  t]),
  normalize([ 0,  1,  t]),
  normalize([ 0, -1, -t]),
  normalize([ 0,  1, -t]),

  normalize([ t,  0, -1]),
  normalize([ t,  0,  1]),
  normalize([-t,  0, -1]),
  normalize([-t,  0,  1]),
];

export const ICOSAHEDRON_FACES: [number, number, number][] = [
  [0, 11, 5],
  [0, 5, 1],
  [0, 1, 7],
  [0, 7, 10],
  [0, 10, 11],
  [1, 5, 9],
  [5, 11, 4],
  [11, 10, 2],
  [10, 7, 6],
  [7, 1, 8],
  [3, 9, 4],
  [3, 4, 2],
  [3, 2, 6],
  [3, 6, 8],
  [3, 8, 9],
  [4, 9, 5],
  [2, 4, 11],
  [6, 2, 10],
  [8, 6, 7],
  [9, 8, 1],
];

export function pointOnFace(
  face: number,
  u: number,
  v: number,
  n: number
): Vec3 {
  const [i0, i1, i2] = ICOSAHEDRON_FACES[face];
  const v0 = ICOSAHEDRON_VERTICES[i0];
  const v1 = ICOSAHEDRON_VERTICES[i1];
  const v2 = ICOSAHEDRON_VERTICES[i2];

  const w = n - u - v;

  const p = add(
    scale(v0, w / n),
    add(scale(v1, u / n), scale(v2, v / n))
  );

  return normalize(p);
}
