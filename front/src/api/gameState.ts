export interface UnitDTO {
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

export async function loadGameState(playerId: number): Promise<UnitDTO[]> {
  const res = await fetch(`/api/state/${playerId}`);
  console.log(res.status, res.headers.get("content-type"));
  if (!res.ok) {
    throw new Error("Failed to load game state");
  }
  return res.json();
}
