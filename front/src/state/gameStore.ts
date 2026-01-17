// src/state/gameStore.ts
import { createStore } from "solid-js/store"
import { TileWithUnits } from "../types/game"

export const [gameState, setGameState] = createStore({
  selectedTile: null as TileWithUnits | null,
  tiles: [] as TileWithUnits[],
})
