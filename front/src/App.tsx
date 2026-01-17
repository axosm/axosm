// src/App.tsx
import { onMount } from "solid-js"
import * as THREE from "three"
import { createRenderer } from "./three/renderer"
import { createGrid } from "./three/grid"
import { setupPicking } from "./three/input"
import { UnitPopup } from "./ui/UnitPopup"

export default function App() {
  let container!: HTMLDivElement

  onMount(() => {
    const scene = new THREE.Scene()
    const camera = new THREE.OrthographicCamera(0, 10, 10, 0, 0.1, 100)
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
