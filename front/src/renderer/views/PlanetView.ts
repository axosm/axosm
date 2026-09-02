// renderer/views/PlanetView.ts
import * as THREE from 'three';
import { BaseView } from './BaseView';
import { GameState, Unit } from '../../api/api';
import { getTileCenter, getTileVertices } from '../math/GoldbergUtils';
import { CameraController } from '../CameraController';

export class PlanetView extends BaseView {
  private unitsGroup = new THREE.Group();
  private tileGroup = new THREE.Group();
  private planetRadius = 5;
  private subdivision = 16; // Adjusted to accommodate u=0, v=13
  private cameraController: CameraController;

  constructor(cameraController: CameraController) {
    super();this.cameraController = cameraController;
    this.container.add(this.tileGroup);
    this.container.add(this.unitsGroup);
  }

  public onEnter(data: GameState): void {
    if (!data || !Array.isArray(data.units) || data.units.length === 0) return;

    // Focus on the first unit's tile
    const primaryUnit = data.units[0];
    if (
      primaryUnit.location_mode === 'planet_surface' &&
      typeof primaryUnit.planet_face === 'number' &&
      typeof primaryUnit.planet_u === 'number' &&
      typeof primaryUnit.planet_v === 'number'
    ) {
      const tileCenter = this.renderSingleTile(
        primaryUnit.planet_face,
        primaryUnit.planet_u,
        primaryUnit.planet_v
      );

      // Focus camera target directly onto the tile center, setting zoom radius close up
      if (tileCenter) {
        this.cameraController.setTarget(tileCenter, 6.5);
      }
    }

    this.renderUnits(data.units);
  }

  private renderSingleTile(face: number, u: number, v: number): THREE.Vector3 {
    // Clear old tile mesh if re-entered
    while (this.tileGroup.children.length > 0) {
      this.tileGroup.remove(this.tileGroup.children[0]);
    }

    const tileCenter = getTileCenter(face, u, v, this.subdivision, this.planetRadius);
    const boundaryPoints = getTileVertices(face, u, v, this.subdivision, this.planetRadius);

    // Build triangulated geometry from center out to boundary vertices (Fan layout)
    const positions: number[] = [];
    for (let i = 0; i < boundaryPoints.length; i++) {
      const nextIdx = (i + 1) % boundaryPoints.length;
      
      // Triangle: Center -> Boundary[i] -> Boundary[i+1]
      positions.push(
        tileCenter.x, tileCenter.y, tileCenter.z,
        boundaryPoints[i].x, boundaryPoints[i].y, boundaryPoints[i].z,
        boundaryPoints[nextIdx].x, boundaryPoints[nextIdx].y, boundaryPoints[nextIdx].z
      );
    }

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geometry.computeVertexNormals();

    const material = new THREE.MeshStandardMaterial({
      color: 0x2e8b57,
      side: THREE.DoubleSide,
      roughness: 0.8,
    });

    const tileMesh = new THREE.Mesh(geometry, material);

    // Add wireframe outline around tile edge
    const lineGeo = new THREE.BufferGeometry().setFromPoints([...boundaryPoints, boundaryPoints[0]]);
    const lineMat = new THREE.LineBasicMaterial({ color: 0x00ffcc, linewidth: 2 });
    const wireframe = new THREE.Line(lineGeo, lineMat);

    this.tileGroup.add(tileMesh);
    this.tileGroup.add(wireframe);

    return tileCenter;
  }

  public renderUnits(units: Unit[]): void {
    while (this.unitsGroup.children.length > 0) {
      this.unitsGroup.remove(this.unitsGroup.children[0]);
    }

    units.forEach((unit) => {
      if (
        unit.location_mode === 'planet_surface' &&
        typeof unit.planet_face === 'number' &&
        typeof unit.planet_u === 'number' &&
        typeof unit.planet_v === 'number'
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

        // Align unit height axis along sphere surface normal
        const normal = position.clone().normalize();
        unitMesh.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), normal);

        this.unitsGroup.add(unitMesh);
      }
    });
  }

  private createUnitMarker(unit: Unit): THREE.Mesh {
    const geometry = new THREE.CylinderGeometry(0.08, 0.2, 0.5, 8);
    geometry.translate(0, 0.25, 0); // Position base on tile surface
    const material = new THREE.MeshStandardMaterial({ color: 0xffaa00 });
    return new THREE.Mesh(geometry, material);
  }

  public update(delta: number): void {}
  public onLeave(): void {}
}