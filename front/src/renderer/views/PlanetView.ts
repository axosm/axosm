// renderer/views/PlanetView.ts
import * as THREE from 'three';
import { BaseView } from './BaseView';
import { GameState, Unit, PlanetTile } from '../../api/api';
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
    if (!data) return;

    // 1. Render all tiles provided in the state
    if (Array.isArray(data.tiles)) {
      this.renderTiles(data.tiles);
    }

    // 2. Focus camera on the primary unit if available
    if (Array.isArray(data.units) && data.units.length > 0) {
      const primaryUnit = data.units[0];
      if (
        primaryUnit.location_mode === 'planet_surface' &&
        typeof primaryUnit.planet_face === 'number' &&
        typeof primaryUnit.planet_u === 'number' &&
        typeof primaryUnit.planet_v === 'number'
      ) {
        const tileCenter = getTileCenter(
          primaryUnit.planet_face,
          primaryUnit.planet_u,
          primaryUnit.planet_v,
          this.subdivision,
            this.planetRadius
        );
        this.cameraController.setTarget(tileCenter, 5.0);
      }
      this.renderUnits(data.units);
    }
  }

  public renderTiles(tiles: PlanetTile[]): void {
    // Clear old tiles
    while (this.tileGroup.children.length > 0) {
      this.tileGroup.remove(this.tileGroup.children[0]);
    }

    tiles.forEach((tile) => {
      const tileCenter = getTileCenter(tile.face, tile.u, tile.v, this.subdivision, this.planetRadius);
      const boundaryPoints = getTileVertices(tile.face, tile.u, tile.v, this.subdivision, this.planetRadius);

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

      // Customize color based on tile properties
      let tileColor = 0x2e8b57; // Default Sea/Standard color
      if (tile.tile_type === 'plains') {
        tileColor = 0x558b2f; 
      }
      
      // Highlight owned tiles (e.g., player territory tint)
      if (tile.owner_player_id !== null) {
        tileColor = 0x1976d2; // Blue tint for player-owned territory
      }

      const material = new THREE.MeshStandardMaterial({
        color: tileColor,
        side: THREE.DoubleSide,
        roughness: 0.8,
      });

      const tileMesh = new THREE.Mesh(geometry, material);

      // Outline wireframe
      const lineGeo = new THREE.BufferGeometry().setFromPoints([...boundaryPoints, boundaryPoints[0]]);
      const lineMat = new THREE.LineBasicMaterial({ 
        color: tile.owner_player_id !== null ? 0x64b5f6 : 0x00ffcc 
      });
      const wireframe = new THREE.Line(lineGeo, lineMat);

      this.tileGroup.add(tileMesh);
      this.tileGroup.add(wireframe);
    });
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

bug on overlapping tiles
https://gemini.google.com/app/15eb5c73fedd0c1f