// src/three/grid.ts
import * as THREE from "three"
import { fetchTile } from "../api/gameApi"
import { setGameState } from "../state/gameStore"

export function createGrid(scene: THREE.Scene) {
  const size = 10

  for (let x = 0; x < size; x++) {
    for (let y = 0; y < size; y++) {
      const geom = new THREE.PlaneGeometry(1, 1)
      const mat = new THREE.MeshBasicMaterial({
        color: (x + y) % 2 === 0 ? 0x444444 : 0x555555,
        side: THREE.DoubleSide,
      })
      const tile = new THREE.Mesh(geom, mat)

      tile.position.set(x, y, 0)
      tile.userData.tileId = `${x}-${y}`
      scene.add(tile)
    }
  }
}
