import { api, GameState } from "./api/api";
import { GameRenderer } from "./renderer/GameRenderer";

class Game {
  private renderer!: GameRenderer;

  private gameState: GameState | null = null;
  
  private pendingTimers: Map<string, ReturnType<typeof setTimeout>> = new Map();
  private battleSSE: EventSource | null = null;


  constructor() {
  
  }


  async boot() {
    this.gameState = await api.getGameState();

    console.log(this.gameState);

    const container = document.getElementById('game-canvas')!;
    this.renderer = new GameRenderer(container);
  }

}

// ── Bootstrap ──────────────────────────────────────────────────
const game = new Game();
game.boot();


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