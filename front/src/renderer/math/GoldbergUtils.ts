import * as THREE from 'three';

// Standard normalized icosahedron vertices
const PHI = (1 + Math.sqrt(5)) / 2;
const ICO_VERTICES: THREE.Vector3[] = [
  new THREE.Vector3(-1,  PHI,  0).normalize(),
  new THREE.Vector3( 1,  PHI,  0).normalize(),
  new THREE.Vector3(-1, -PHI,  0).normalize(),
  new THREE.Vector3( 1, -PHI,  0).normalize(),
  new THREE.Vector3( 0, -1,  PHI).normalize(),
  new THREE.Vector3( 0,  1,  PHI).normalize(),
  new THREE.Vector3( 0, -1, -PHI).normalize(),
  new THREE.Vector3( 0,  1, -PHI).normalize(),
  new THREE.Vector3( PHI,  0, -1).normalize(),
  new THREE.Vector3( PHI,  0,  1).normalize(),
  new THREE.Vector3(-PHI,  0, -1).normalize(),
  new THREE.Vector3(-PHI,  0,  1).normalize(),
];

// The 20 triangular faces of an icosahedron (indices to ICO_VERTICES)
const ICO_FACES: [number, number, number][] = [
  [0, 11, 5],  [0, 5, 1],   [0, 1, 7],   [0, 7, 10],  [0, 10, 11],
  [1, 5, 9],   [5, 11, 4],  [11, 10, 2], [10, 7, 6],  [7, 1, 8],
  [3, 9, 4],   [3, 4, 2],   [3, 2, 6],   [3, 6, 8],   [3, 8, 9],
  [4, 9, 5],   [2, 4, 11],  [6, 2, 10],  [8, 6, 7],   [9, 8, 1]
];

/**
 * Calculates the 3D surface position for a unit on a Goldberg sphere.
 * @param face Face index (0 to 19)
 * @param u Local U coordinate on the face grid
 * @param v Local V coordinate on the face grid
 * @param subdivision Grid resolution N along each face edge
 * @param radius Planet radius in world units
 */
export function getTileCenter(
  face: number,
  u: number,
  v: number,
  subdivision: number,
  radius: number
): THREE.Vector3 {
  const [i0, i1, i2] = ICO_FACES[face];
  const v0 = ICO_VERTICES[i0];
  const v1 = ICO_VERTICES[i1];
  const v2 = ICO_VERTICES[i2];

  // Convert (u, v) grid position to barycentric coordinates (w0, w1, w2)
  const w1 = u / subdivision;
  const w2 = v / subdivision;
  const w0 = 1.0 - w1 - w2;

  // Linear interpolation on the flat face triangle
  const pointOnTriangle = new THREE.Vector3()
    .addScaledVector(v0, w0)
    .addScaledVector(v1, w1)
    .addScaledVector(v2, w2);

  // Normalize to push outwards to the sphere surface, then scale by radius
  return pointOnTriangle.normalize().multiplyScalar(radius);
}