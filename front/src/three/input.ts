// src/three/input.ts
import * as THREE from "three"
import { fetchTile } from "../api/gameApi"
import { setGameState } from "../state/gameStore"

export function setupPicking(
  camera: THREE.Camera,
  scene: THREE.Scene,
  dom: HTMLElement
) {
  const raycaster = new THREE.Raycaster()
  const mouse = new THREE.Vector2()

  dom.addEventListener("click", async (e) => {
    mouse.x = (e.clientX / dom.clientWidth) * 2 - 1
    mouse.y = -(e.clientY / dom.clientHeight) * 2 + 1

    raycaster.setFromCamera(mouse, camera)
    const hits = raycaster.intersectObjects(scene.children)

    if (hits.length) {
      const tileId = hits[0].object.userData.tileId
      const tile = await fetchTile(tileId)
      setGameState("selectedTile", tile)
    }
  })
}
