import * as THREE from 'three';
import { BaseView } from '../base_view';

export interface PlanetData {
  id: string;
  name: string;
  radius: number;
  buildings: Map<string, { type: string; level: number; tileIndex: number }>;
  units?: any[];
}

export class PlanetView extends BaseView {
  private planetMesh!: THREE.Mesh;
  private atmosphereMesh!: THREE.Mesh;
  private buildingGroup: THREE.Group = new THREE.Group();
  private gridGroup: THREE.Group = new THREE.Group();

  private currentPlanetData: PlanetData | null = null;
  private hoveredTileIndex: number | null = null;

  constructor() {
    super(); // Initializes this.container (THREE.Group)
    this.setupBaseMeshes();
  }

  private setupBaseMeshes(): void {
    // 1. Root group for all surface objects
    this.container.add(this.buildingGroup);
    this.container.add(this.gridGroup);

    // 2. Base planetary surface geometry & material
    const geometry = new THREE.SphereGeometry(100, 64, 64);
    const material = new THREE.MeshStandardMaterial({
      color: 0x2b558c,
      roughness: 0.8,
    });
    
    this.planetMesh = new THREE.Mesh(geometry, material);
    this.container.add(this.planetMesh);

    // 3. Atmosphere layer shader/mesh
    const atmosGeometry = new THREE.SphereGeometry(102, 64, 64);
    const atmosMaterial = new THREE.MeshBasicMaterial({
      color: 0x418bd4,
      transparent: true,
      opacity: 0.25,
      side: THREE.BackSide,
    });
    this.atmosphereMesh = new THREE.Mesh(atmosGeometry, atmosMaterial);
    this.container.add(this.atmosphereMesh);
  }

  // --- View Lifecycle Hooks ---

  public override onEnter(planetData?: PlanetData): void {
    this.container.visible = true;

    if (planetData) {
      this.currentPlanetData = planetData;
      this.loadPlanetSurface(planetData);
    }
  }

  public override onLeave(): void {
    this.container.visible = false;
    this.clearSurface();
  }

  // --- Surface & Building Logic ---

  private loadPlanetSurface(data: PlanetData): void {
    this.clearSurface();

    // Generate/render buildable surface grid
    this.renderGridOverlay();

    // Populate existing buildings from data
    data.buildings.forEach((building) => {
      this.spawnBuildingMesh(building.type, building.tileIndex);
    });
  }

  private renderGridOverlay(): void {
    // Generate hex or quad grid overlay over planet surface for tile selection
  }

  private spawnBuildingMesh(buildingType: string, tileIndex: number): void {
    // Place 3D building model onto calculated tile coordinates
  }

  private clearSurface(): void {
    // Clear building/grid meshes when switching planets or zooming out
    while (this.buildingGroup.children.length > 0) {
      const child = this.buildingGroup.children[0];
      this.buildingGroup.remove(child);
    }
  }

  // --- Raycasting & Interactivity ---

  public onPointerMove(raycaster: THREE.Raycaster): void {
    // Highlight hovered tile/hex for building placement or inspection
    const intersects = raycaster.intersectObject(this.planetMesh);
    if (intersects.length > 0) {
      // Calculate tile ID from hit point
    }
  }

  public override update(delta: number): void {
    // Rotate clouds/atmosphere or animate surface units
    if (this.atmosphereMesh) {
      this.atmosphereMesh.rotation.y += delta * 0.02;
    }
  }
}