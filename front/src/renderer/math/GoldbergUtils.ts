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

/**
 * Checks if a grid coordinate represents one of the 12 pentagon centers.
 */
export function isPentagonTile(u: number, v: number, subdivision: number): boolean {
  return (
    (u === 0 && v === 0) ||
    (u === subdivision && v === 0) ||
    (u === 0 && v === subdivision)
  );
}

/**
 * Generates boundary 3D vertices for a single Goldberg tile on the sphere surface.
 */
export function getTileVertices(
  face: number,
  u: number,
  v: number,
  subdivision: number,
  radius: number
): THREE.Vector3[] {
  const isPentagon = isPentagonTile(u, v, subdivision);
  const numSides = isPentagon ? 5 : 6;
  const center = getTileCenter(face, u, v, subdivision, radius);

  // Derive an orthonormal basis on the tangent plane at the tile center
  const normal = center.clone().normalize();
  let up = Math.abs(normal.y) > 0.99 ? new THREE.Vector3(1, 0, 0) : new THREE.Vector3(0, 1, 0);
  const tangentX = new THREE.Vector3().crossVectors(up, normal).normalize();
  const tangentY = new THREE.Vector3().crossVectors(normal, tangentX).normalize();

  // Estimate tile radius based on planet radius and grid subdivision scale
  const tileRadius = (radius * (Math.PI / 2)) / (subdivision * 1.5);

  const vertices: THREE.Vector3[] = [];
  for (let i = 0; i < numSides; i++) {
    const angle = (i * 2 * Math.PI) / numSides;
    const x = Math.cos(angle) * tileRadius;
    const y = Math.sin(angle) * tileRadius;

    // Offset point from center along local tangent plane and project back to sphere radius
    const vertex = center
      .clone()
      .addScaledVector(tangentX, x)
      .addScaledVector(tangentY, y)
      .normalize()
      .multiplyScalar(radius);

    vertices.push(vertex);
  }

  return vertices;
}