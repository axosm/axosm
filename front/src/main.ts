import { api, GameState } from "./api/api";
import { GameRenderer } from "./renderer/GameRenderer";
import { CameraController } from "./renderer/CameraController";
import { TransitionManager } from "./renderer/TransitionManager";
import { BaseView } from "./renderer/views/BaseView";
import {UniverseView} from "./renderer/views/UniverseView";
import {GalaxyView} from "./renderer/views/GalaxyView";      
import {SystemView} from "./renderer/views/SystemView";      
import {PlanetView} from "./renderer/views/PlanetView";      
      

export enum ViewMode {
  UNIVERSE,
  GALAXY,
  SYSTEM,
  PLANET
}


class App {
  private gameRenderer!: GameRenderer;
  private cameraController!: CameraController;
  private transitionManager!: TransitionManager;

  private gameState: GameState | null = null;

  private pendingTimers: Map<string, ReturnType<typeof setTimeout>> = new Map();
  private battleSSE: EventSource | null = null;

  private views: Map<ViewMode, BaseView> = new Map();
  private activeViewMode: ViewMode = ViewMode.PLANET;
  
  constructor() {

    this.init();
  }

  // private async init(): Promise<void> {
  //   this.gameState = await api.getGameState();

  //   console.log(this.gameState);

  //   const container = document.getElementById("game-canvas")!;
  //   this.renderer = new GameRenderer(container);
  // }


  // 1. Marked as async to fetch initial data before initializing the game
  private async init(): Promise<void> {
    // Fetch state from server FIRST
    this.gameState = await api.getGameState();
    console.log('Game state loaded:', this.gameState);

    // Setup WebGL rendering and camera

    const container = document.getElementById("game-canvas")!;
    this.gameRenderer = new GameRenderer(container);
    this.cameraController = new CameraController(this.gameRenderer.camera);
    
    // Instantiate view layers
    this.views = new Map<ViewMode, BaseView>([
      [ViewMode.UNIVERSE, new UniverseView()],
      [ViewMode.GALAXY, new GalaxyView()],
      [ViewMode.SYSTEM, new SystemView()],
      [ViewMode.PLANET, new PlanetView()]
    ]);

    this.transitionManager = new TransitionManager(this.cameraController);

    // Register all view root containers in the main scene graph
    // TODO adding all views might not be a good idea
    this.views.forEach((view, mode) => {
      view.container.visible = (mode === this.activeViewMode);
      this.gameRenderer.scene.add(view.container);
    });

    this.bindEvents();

    // Pass fetched game state context to starting view
    const initialView = this.views.get(this.activeViewMode)!;
    initialView.onEnter(this.gameState);

    // Start the frame loop
    this.startLoop();
  }

  // Handle smooth transitions between navigation scales
  public transitionTo(targetMode: ViewMode, contextData?: any): void {
    if (targetMode === this.activeViewMode) return;

    const currentView = this.views.get(this.activeViewMode)!;
    const targetView = this.views.get(targetMode)!;

    this.transitionManager.startTransition(
      currentView,
      targetView,
      contextData,
      () => {
        currentView.onLeave();
        targetView.onEnter(contextData);
        this.activeViewMode = targetMode;
      }
    );
  }

  private startLoop(): void {
    let lastTime = performance.now();

    const animate = (currentTime: number) => {
      const delta = (currentTime - lastTime) / 1000;
      lastTime = currentTime;

      // should we pass delta?
      // this.cameraController.update(delta);
      this.cameraController.update();
      this.transitionManager.update(delta);
      
      const currentView = this.views.get(this.activeViewMode);
      if (currentView) {
        currentView.update(delta);
      }

      this.gameRenderer.render();

      requestAnimationFrame(animate);
    };

    requestAnimationFrame(animate);
  }

  private bindEvents(): void {
    window.addEventListener('resize', () => this.gameRenderer.onResize());
    
    // Only planet view atm
    // this.cameraController.onZoomThresholdExceeded((direction) => {
    //   if (direction === 'out' && this.activeViewMode === ViewMode.PLANET) {
    //     this.transitionTo(ViewMode.SYSTEM);
    //   } else if (direction === 'in' && this.activeViewMode === ViewMode.SYSTEM) {
    //     this.transitionTo(ViewMode.PLANET);
    //   }
    // });
  }


}

new App();

// // Called when server confirms a move, returns arrival_time
// private scheduleMovepoll(unit: Unit, arrivalTime: string) {
//   const delay = new Date(arrivalTime).getTime() - Date.now();

//   const timer = setTimeout(async () => {
//     this.pendingTimers.delete(unit.id);
//     const fresh = await api.getGameState();
//     this.applyState(fresh);
//   }, delay);

//   this.pendingTimers.set(unit.id, timer);
// }

// // Same pattern for construction / research
// private scheduleCompletionPoll(jobId: string, completesAt: string) {
//   const delay = new Date(completesAt).getTime() - Date.now();

//   const timer = setTimeout(async () => {
//     this.pendingTimers.delete(jobId);
//     const fresh = await api.getGameState();
//     this.applyState(fresh);
//   }, delay);

//   this.pendingTimers.set(jobId, timer);
// }

// private connectBattleSSE() {
//   this.battleSSE = new EventSource('/api/battle-alerts');

//   this.battleSSE.addEventListener('combat_tick', (e) => {
//     const data = JSON.parse(e.data);
//     this.hud.showCombatAlert(data);
//     this.renderer.updateUnits(data.units_on_tile);
//   });

//   this.battleSSE.addEventListener('unit_destroyed', (e) => {
//     const data = JSON.parse(e.data);
//     this.renderer.removeUnit(data.unit_id);
//     this.hud.renderUnits(this.gameState!.units.filter(u => u.id !== data.unit_id));
//   });

//   this.battleSSE.onerror = () => {
//     // SSE auto-reconnects, but you can add backoff logic here
//   };
// }

//   private applyState(state: GameState) {
//   this.gameState = state;
//   this.renderer.loadTiles(state.visible_tiles, state.subdivision);
//   this.renderer.placeUnits(state.units, state.subdivision, state.planet_id);
//   this.renderer.drawMinimap(...);
//   this.hud.renderUnits(state.units);
//   // re-register pending timers that survived the refresh
//   // (in case the page was reloaded mid-movement)
//   for (const unit of state.units) {
//     if (unit.arrival_time && !this.pendingTimers.has(unit.id)) {
//       this.scheduleMovepoll(unit, unit.arrival_time);
//     }
//   }
// }
