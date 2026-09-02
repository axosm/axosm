// renderer/CameraController.ts
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';

export class CameraController {
  public camera: THREE.PerspectiveCamera;
  public controls: OrbitControls;

  constructor(camera: THREE.PerspectiveCamera, canvas: HTMLElement) {
    this.camera = camera;
    this.controls = new OrbitControls(this.camera, canvas);

    this.configureControls();
  }

  private configureControls(): void {
    // Force panning parallel to ground plane (Humankind style)
    this.controls.screenSpacePanning = false;

    // Enable smooth physics momentum / inertia
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.08;

    // Panning & Rotation speeds
    this.controls.rotateSpeed = 0.8;
    this.controls.panSpeed = 1.0;
    this.controls.zoomSpeed = 1.2;

    // Distance bounds (zoom in / zoom out)
    this.controls.minDistance = 1.5;
    this.controls.maxDistance = 15.0;

    // Tilt limits: Keep the camera angled like Humankind (~30° to ~75°)
    this.controls.minPolarAngle = Math.PI / 6;   // ~30 degrees
    this.controls.maxPolarAngle = Math.PI / 2.3; // ~78 degrees

    // Humankind Mouse Bindings: Left-click Pan, Right-click Rotate, Scroll Zoom
    this.controls.mouseButtons = {
      LEFT: THREE.MOUSE.PAN,
      MIDDLE: THREE.MOUSE.DOLLY,
      RIGHT: THREE.MOUSE.ROTATE,
    };

    // Keyboard support: WASD panning via OrbitControls keys
    this.controls.listenToKeyEvents(window);
    this.controls.keyPanSpeed = 15.0;
  }

  /**
   * Centers and focuses the camera target onto a specific 3D tile/unit
   */
  public setTarget(newTarget: THREE.Vector3, distance?: number): void {
    this.controls.target.copy(newTarget);

    if (distance !== undefined) {
      const offset = new THREE.Vector3()
        .subVectors(this.camera.position, this.controls.target)
        .normalize()
        .multiplyScalar(distance);

      this.camera.position.copy(this.controls.target).add(offset);
    }

    this.controls.update();
  }

  /**
   * Re-synchronizes control target after smooth camera transitions
   */
  public syncFromCurrentPosition(): void {
    this.controls.update();
  }

  /**
   * Called every frame in startLoop()
   */
  public update(delta: number = 0.016): void {
    // Required every frame when enableDamping is true
    this.controls.update();
  }
}