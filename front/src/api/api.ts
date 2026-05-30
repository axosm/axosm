// All types mirror the Rust models

export interface AuthResponse {
  token: string;
  player_id: number;
  username: string;
}

export interface Unit {
  id: number;
  unit_type: string;
  hp: number;
  player_id: number;
  location_mode: string;
  planet_id: number | null;
  planet_face: number | null;
  planet_u: number | null;
  planet_v: number | null;
}

export interface PlanetTile {
  id: number;
  planet_id: number;
  face: number;
  u: number;
  v: number;
  tile_type: string;
  yield_quality: number;
  rare_deposit: string | null;
  owner_player_id: number | null;
  explored: boolean;
}

export interface GameState {
  player_id: number;
  // planet_id: number;
  // galaxy_id: number;
  // system_id: number;
  // planet_seed: number;
  // subdivision: number;
  units: Unit[];
  visible_tiles: PlanetTile[];
}

export interface InitResponse {
  state: GameState;
  is_new_player: boolean;
}

// ── API client ────────────────────────────────────────────────

const BASE = "/api";
const IS_LOCAL = import.meta.env.VITE_GAME_MODE === "local";

export class ApiClient {
  private credential: string | null = null;

  constructor() {
    this.initCredentials();
  }

  private initCredentials() {
    if (IS_LOCAL) {
      // Local Mode: Fallback straight to an automated local UUID device key
      this.credential = localStorage.getItem("space4x_session_key");
      if (!this.credential) {
        this.credential = crypto.randomUUID();
        localStorage.setItem("space4x_session_key", this.credential);
      }
    } else {
      // Production Mode: Fetch JWT matching token key
      this.credential = localStorage.getItem("space4x_token");
    }
  }

  // Administers tokens after user successfully logins in Production
  setToken(t: string) {
    if (IS_LOCAL) return;
    this.credential = t;
    localStorage.setItem("space4x_token", t);
  }

  clearToken() {
    this.credential = null;
    localStorage.removeItem("space4x_token");
    localStorage.removeItem("space4x_session_key");
  }

  private headers(): HeadersInit {
    const h: Record<string, string> = { "Content-Type": "application/json" };

    if (this.credential) {
      if (IS_LOCAL) {
        // Local mode compilation output sends custom session string header
        h["X-Session-Key"] = this.credential;
      } else {
        // Production compilation output sends Bearer Authorization format
        h["Authorization"] = `Bearer ${this.credential}`;
      }
    }
    return h;
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
  ): Promise<T> {
    const res = await fetch(BASE + path, {
      method,
      headers: this.headers(),
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error(text || res.statusText);
    }
    return res.json() as Promise<T>;
  }

  // ── Authentication Endpoints ───────────────────────────────

  async register(
    username: string,
    email: string,
    password: string,
  ): Promise<AuthResponse> {
    if (IS_LOCAL) throw new Error("Registration disabled in Local play.");
    return this.request("POST", "/auth/register", {
      username,
      email,
      password,
    });
  }

  async login(email: string, password: string): Promise<AuthResponse> {
    if (IS_LOCAL) throw new Error("Login disabled in Local play.");
    return this.request("POST", "/auth/login", { email, password });
  }

  // ── Core Game State Engine ──────────────────────────────────

  async getGameState(): Promise<GameState> {
    return this.request("GET", "/state");
  }

  async moveUnit(unit_id: number, to_face: number, to_u: number, to_v: number) {
    return this.request("POST", `/api/units/${unit_id}/move`, {
      to_face,
      to_u,
      to_v,
    });
  }
}

export const api = new ApiClient();
