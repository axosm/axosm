// src/types/game.ts
export type Tile = {
  id: string
  x: number
  y: number
}

export type Unit = {
  id: string
  name: string
  strength: number
}

export type TileWithUnits = Tile & {
  units: Unit[]
}
