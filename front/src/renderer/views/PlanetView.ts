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
  private subdivision = 16;
  private cameraController: CameraController;

  constructor(cameraController: CameraController) {
    super();
    this.cameraController = cameraController;
    this.container.add(this.tileGroup);
    this.container.add(this.unitsGroup);
  }

  public onEnter(data: GameState): void {
    if (!data || !Array.isArray(data.units) || data.units.length === 0) return;

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

      // Focus camera pivot directly on the unit tile center
      if (tileCenter) {
        this.cameraController.setTarget(tileCenter, 5.0);
      }
    }

    this.renderUnits(data.units);
  }

  private renderSingleTile(face: number, u: number, v: number): THREE.Vector3 {
    while (this.tileGroup.children.length > 0) {
      this.tileGroup.remove(this.tileGroup.children[0]);
    }

    const tileCenter = getTileCenter(face, u, v, this.subdivision, this.planetRadius);
    const boundaryPoints = getTileVertices(face, u, v, this.subdivision, this.planetRadius);

    const positions: number[] = [];
    for (let i = 0; i < boundaryPoints.length; i++) {
      const nextIdx = (i + 1) % boundaryPoints.length;
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

    const lineGeo = new THREE.BufferGeometry().setFromPoints([...boundaryPoints, boundaryPoints[0]]);
    const lineMat = new THREE.LineBasicMaterial({ color: 0x00ffcc });
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

        const normal = position.clone().normalize();
        unitMesh.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), normal);

        this.unitsGroup.add(unitMesh);
      }
    });
  }

  private createUnitMarker(unit: Unit): THREE.Mesh {
    const geometry = new THREE.CylinderGeometry(0.08, 0.2, 0.5, 8);
    geometry.translate(0, 0.25, 0);
    const material = new THREE.MeshStandardMaterial({ color: 0xffaa00 });
    return new THREE.Mesh(geometry, material);
  }

  public update(delta: number): void {}
  public onLeave(): void {}
}