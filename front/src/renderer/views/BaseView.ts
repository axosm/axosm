import * as THREE from 'three';

export abstract class BaseView {
  // Every view owns a root 3D container group
  public container: THREE.Group;

  constructor() {
    this.container = new THREE.Group();
  }

  /**
   * Called when entering this scale level.
   * Use this to make meshes visible, bind scale-specific listeners, or parse state data.
   */
  public onEnter(contextData?: any): void {
    this.container.visible = true;
  }

  /**
   * Called when leaving this scale level.
   * Use this to hide meshes, pause animations, or unbind local event listeners.
   */
  public onLeave(): void {
    this.container.visible = false;
  }

  /**
   * Called on every frame from the main loop in main.ts.
   * Override in child classes to animate local scene elements (e.g., rotating planets, moving ships).
   */
  public abstract update(delta: number): void;
}