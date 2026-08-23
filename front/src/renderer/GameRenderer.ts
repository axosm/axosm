// renderer/GameRenderer.ts
import * as THREE from 'three';

export class GameRenderer {
  camera: THREE.PerspectiveCamera;
  renderer: THREE.WebGLRenderer;
  scene: THREE.Scene;

constructor(container: HTMLElement) {
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x020208);

    // Get width and height from container bounds instead of window directly
    const width = container.clientWidth || window.innerWidth;
    const height = container.clientHeight || window.innerHeight;

    this.camera = new THREE.PerspectiveCamera(60, width / height, 0.1, 1000);
    this.camera.position.set(0, 0, 15);
    this.camera.lookAt(0, 0, 0);

    this.renderer = new THREE.WebGLRenderer({
      canvas: container as HTMLCanvasElement,
      antialias: true,
    });
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

    // Append WebGL DOM canvas element into container
    // container.appendChild(this.renderer.domElement);

    this.setupLighting();
  }

  private setupLighting(): void {
    const ambient = new THREE.AmbientLight(0xffffff, 0.6);
    const sun = new THREE.DirectionalLight(0xffffff, 1.5);
    sun.position.set(10, 20, 15);

    this.scene.add(ambient);
    this.scene.add(sun);
  }

  onResize(): void {
    const width = window.innerWidth;
    const height = window.innerHeight;
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);
  }

  render(): void {
    this.renderer.render(this.scene, this.camera);
  }
}

// // Resize
// window.addEventListener("resize", () => {
//   camera.aspect = window.innerWidth / window.innerHeight;
//   camera.updateProjectionMatrix();
//   renderer.setSize(window.innerWidth, window.innerHeight);
// });


// // aaaa https://claude.ai/share/34acb4d5-73a6-482c-b505-0049de5d6e26

// const canvas = document.getElementById("game-canvas") as HTMLCanvasElement;

// // Scene
// const scene = new THREE.Scene();
// scene.background = new THREE.Color(0x000010);

// Camera
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
