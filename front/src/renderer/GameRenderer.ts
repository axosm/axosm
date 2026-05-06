import * as THREE from 'three';

export class GameRenderer {

  constructor(container: HTMLElement) {

  }
}


// // aaaa https://claude.ai/share/34acb4d5-73a6-482c-b505-0049de5d6e26

// const canvas = document.getElementById("game-canvas") as HTMLCanvasElement;

// // Scene
// const scene = new THREE.Scene();
// scene.background = new THREE.Color(0x000010);

// // Camera
// const camera = new THREE.PerspectiveCamera(
//   75,
//   window.innerWidth / window.innerHeight,
//   0.1,
//   1000,
// );
// camera.position.z = 5;

// // Renderer
// const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
// renderer.setSize(window.innerWidth, window.innerHeight);
// renderer.setPixelRatio(window.devicePixelRatio);

// // A simple sphere (planet)
// const geometry = new THREE.SphereGeometry(1, 32, 32);
// const material = new THREE.MeshStandardMaterial({ color: 0x4488ff });
// const planet = new THREE.Mesh(geometry, material);
// scene.add(planet);

// // Lights
// scene.add(new THREE.AmbientLight(0xffffff, 0.3));
// const sun = new THREE.PointLight(0xffffff, 2);
// sun.position.set(10, 10, 10);
// scene.add(sun);

// // Resize
// window.addEventListener("resize", () => {
//   camera.aspect = window.innerWidth / window.innerHeight;
//   camera.updateProjectionMatrix();
//   renderer.setSize(window.innerWidth, window.innerHeight);
// });

// // Loop
// function animate() {
//   requestAnimationFrame(animate);
//   planet.rotation.y += 0.005;
//   renderer.render(scene, camera);
// }
// animate();
