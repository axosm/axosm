// renderer/views/PlanetView.ts
import * as THREE from 'three';
import { BaseView } from './BaseView';
import { GameState, Unit } from '../../api/api';
import { getTileCenter } from '../math/GoldbergUtils';

export class PlanetView extends BaseView {
  private planetMesh!: THREE.Mesh;
  private unitsGroup = new THREE.Group();
  private planetRadius = 5;
  private subdivision = 8; // Adjust based on your Goldberg spec

  constructor() {
    super();
    this.container.add(this.unitsGroup);
    this.createPlaceholderPlanet();
  }

  private createPlaceholderPlanet() {
    // Basic planet sphere until tile rendering logic is wired up
    const geom = new THREE.IcosahedronGeometry(this.planetRadius, 4);
    const mat = new THREE.MeshStandardMaterial({
      color: 0x224488,
      wireframe: true,
    });
    this.planetMesh = new THREE.Mesh(geom, mat);
    this.container.add(this.planetMesh);
  }

  public onEnter(data: GameState): void {
    if (!data) return;
    this.renderUnits(data.units);
  }

  public renderUnits(units: Unit[]): void {
  while (this.unitsGroup.children.length > 0) {
    this.unitsGroup.remove(this.unitsGroup.children[0]);
  }

  if (!units) return;

  units.forEach((unit) => {
    if (
      unit.location_mode === 'planet_surface' &&
      unit.planet_face !== null &&
      unit.planet_face >= 0 &&
      unit.planet_face < 20 &&
      unit.planet_u !== null &&
      unit.planet_v !== null
    ) {
      const position = getTileCenter(
        unit.planet_face,
        unit.planet_u,
        unit.planet_v,
        this.subdivision,
        this.planetRadius
      );

      const unitMesh = this.createUnitMarker(unit);
      unitMesh.position.copy(position);

      const normal = position.clone().normalize();
      unitMesh.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), normal);

      this.unitsGroup.add(unitMesh);
    }
  });
}

  private createUnitMarker(unit: Unit): THREE.Mesh {
    // Simple marker representing a unit (e.g., Civ-style pawn)
    const geometry = new THREE.CylinderGeometry(0.05, 0.15, 0.4, 8);
    const material = new THREE.MeshStandardMaterial({ color: 0xffaa00 });
    const mesh = new THREE.Mesh(geometry, material);

    // Offset origin so base of marker sits flush on the terrain surface
    geometry.translate(0, 0.2, 0);
    return mesh;
  }

  public update(delta: number): void {
    // Handle planet rotation or unit animations here
  }

  public onLeave(): void {}
}