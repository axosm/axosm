// src/App.tsx
import { onMount } from "solid-js"
import * as THREE from "three"
import { createRenderer } from "./three/renderer"
import { createGrid } from "./three/grid"
import { setupPicking } from "./three/input"
import { UnitPopup } from "./ui/UnitPopup"
import { fetchUnits } from "./api/gameApi"
import { setGameState } from "./state/gameStore"

export default function App() {
  let container!: HTMLDivElement

  onMount(async () => {
    try {
      // Fetch units with their tile data
      const unitsWithTile = await fetchUnits(1)
      setGameState("units", Array.isArray(unitsWithTile) ? unitsWithTile : [unitsWithTile])
    } catch (error) {
      console.error("Failed to fetch units:", error)
    }

    const scene = new THREE.Scene()
   const camera = new THREE.PerspectiveCamera( 75, window.innerWidth / window.innerHeight, 0.1, 1000 );
    // const camera = new THREE.OrthographicCamera(0, 10, 10, 0, 0.1, 100)
    camera.position.z = 10

    const renderer = createRenderer(container)
    createGrid(scene)
    setupPicking(camera, scene, renderer.domElement)

    const animate = () => {
      requestAnimationFrame(animate)
      renderer.render(scene, camera)
    }
    animate()
  })

  return (
    <>
      <div ref={container} class="w-screen h-screen" />
      <UnitPopup />
    </>
  )
}
