// src/api/gameApi.ts
import { TileWithUnits } from "../types/game"

const API = "http://localhost:3000"

export async function fetchMap(): Promise<TileWithUnits[]> {
  const res = await fetch(`${API}/map/planet/1`)
  return res.json()
}

const USE_FAKE_DATA = true

export async function fetchTile(tileId: string): Promise<TileWithUnits> {
  if (USE_FAKE_DATA) {
    // Simulate network latency
    await new Promise((r) => setTimeout(r, 150))

    const [x, y] = tileId.split("-").map(Number)

    return {
      id: tileId,
      x,
      y,
      units: [
        {
          id: "u1",
          name: "Infantry",
          strength: 10,
        },
        {
          id: "u2",
          name: "Scout",
          strength: 4,
        },
        {
          id: "u3",
          name: "Tank",
          strength: 25,
        },
      ],
    }
  }

  // Real backend call (later)
  const res = await fetch(`http://localhost:3000/tiles/${tileId}`)
  if (!res.ok) {
    throw new Error("Failed to fetch tile")
  }
  return res.json()
}