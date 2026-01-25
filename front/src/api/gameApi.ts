// src/api/gameApi.ts
import { TileWithUnits, UnitWithTile } from "../types/game"
// import { UnitInstance } from "../world/units"

// export async function fetchMap(): Promise<TileWithUnits[]> {
//   const res = await fetch(`/api/map/planet/1`)
//   return res.json()
// }

const USE_FAKE_DATA = true

export async function fetchUnits(playerId: number): Promise<UnitWithTile> {
  // if (USE_FAKE_DATA) {
  //   // Simulate network latency
  //   await new Promise((r) => setTimeout(r, 150))

  //   const [x, y] = tileId.split("-").map(Number)

  //   return {
  //     id: tileId,
  //     x,
  //     y,
  //     units: [
  //       {
  //         id: "u1",
  //         name: "Infantry",
  //         strength: 10,
  //       },
  //       {
  //         id: "u2",
  //         name: "Scout",
  //         strength: 4,
  //       },
  //       {
  //         id: "u3",
  //         name: "Tank",
  //         strength: 25,
  //       },
  //     ],
  //   }
  // }

  // Real backend call (later)
  const res = await fetch(`/api/state/${playerId}`)
  if (!res.ok) {
    throw new Error("Failed to fetch tile")
  }
  return res.json()
}