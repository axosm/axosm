import type { UnitDTO } from "../api/gameState";
import { pointOnFace } from "../math/goldberg";
import type { Vec3 } from "../math/vec3";

const SUBDIVISION_LEVEL = 20;

export interface UnitInstance {
  position: Vec3;
  scale: number;
}

export function buildUnitInstances(units: UnitDTO[]): UnitInstance[] {
  return units.map(u => {
    const p = pointOnFace(
      u.location.face,
      u.location.u,
      u.location.v,
      SUBDIVISION_LEVEL
    );

    return {
      position: p,
      scale: 0.03,
    };
  });
}
