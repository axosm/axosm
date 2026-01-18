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


  id: number;
  player_id: number;
  unit_type: "INFANTRY";
  location: {
    location_type: "PlanetSurface";
    planet_id: number;
    face: number;
    u: number;
    v: number;
  };
}