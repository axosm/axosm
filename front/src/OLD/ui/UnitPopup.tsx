// src/ui/UnitPopup.tsx
import { Show } from "solid-js"
import { gameState } from "../state/gameStore"

export function UnitPopup() {
  return (
    <Show when={gameState.selectedTile}>
      <div class="absolute right-4 top-16 bg-gray-800 text-white p-4 rounded w-64">
        <h2 class="font-bold mb-2">Units</h2>
        <ul>
          {gameState.selectedTile!.units.map(u => (
            <li class="border-b border-gray-600 py-1">
              {u.id} (PLAYER {u.player_id}, TYPE: {u.unit_type})
            </li>
          ))}
        </ul>
      </div>
    </Show>
  )
}
