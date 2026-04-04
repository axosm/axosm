// src/types/game.ts
// export type Tile = {
//   id: string
//   x: number
//   y: number
// }

// export type Unit = {
//   id: string
//   name: string
//   strength: number
// }

// export type TileWithUnits = Tile & {
//   units: Unit[]
// }


export type Tile = {
  planet_id: string
  face: number
  u: number
  v: number
  location_type: "PlanetSurface"
}

export type Unit = {
  id: string
  player_id: number
  unit_type: string
}

export type TileWithUnits = Tile & {
  units: Unit[]
}

export type UnitWithTile = Unit & {
  tile: Tile
}
