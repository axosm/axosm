import * as THREE from 'three';
import { CameraController } from '../scene/camera_controler';
import { BaseView } from './base_view';

export class TransitionManager {
  private cameraController: CameraController;
  private isTransitioning: boolean = false;
  private progress: number = 0; // Ranges from 0.0 (start) to 1.0 (end)
  private duration: number = 1.2; // Transition time in seconds

  private startPos = new THREE.Vector3();
  private targetPos = new THREE.Vector3();
  private fromView?: BaseView;
  private toView?: BaseView;
  private onComplete?: () => void;

  constructor(cameraController: CameraController) {
    this.cameraController = cameraController;
  }

  // 1. Kick off transition between views
  public startTransition(
    fromView: BaseView,
    toView: BaseView,
    targetCameraPos: THREE.Vector3,
    onComplete?: () => void
  ): void {
    this.fromView = fromView;
    this.toView = toView;
    this.onComplete = onComplete;
    this.progress = 0;
    this.isTransitioning = true;

    // Record starting camera position and desired endpoint
    this.startPos.copy(this.cameraController.camera.position);
    this.targetPos.copy(targetCameraPos);

    // Make target view visible alongside current view for rendering
    this.toView.container.visible = true;
  }

  // 2. Called every frame inside main loop
  public update(delta: number): void {
    if (!this.isTransitioning) return;

    this.progress += delta / this.duration;
    const t = Math.min(this.progress, 1.0);

    // Smoothstep easing curve for smooth acceleration/deceleration
    const easeT = t * t * (3 - 2 * t);

    // Move camera towards target position
    this.cameraController.camera.position.lerpVectors(this.startPos, this.targetPos, easeT);

    // Transition complete
    if (t >= 1.0) {
      this.isTransitioning = false;
      if (this.fromView) this.fromView.container.visible = false; // Hide old view
      if (this.onComplete) this.onComplete();
    }
  }

  public get inProgress(): boolean {
    return this.isTransitioning;
  }
}