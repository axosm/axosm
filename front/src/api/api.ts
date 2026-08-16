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
    // 1. Try to load an existing token or session key
    const token = localStorage.getItem("space4x_token");
    let sessionKey = localStorage.getItem("space4x_session_key");

    if (token) {
      this.credential = token;
    } else {
      // 2. Fallback: Generate a session key if none exists
      if (!sessionKey) {
        sessionKey = crypto.randomUUID();
        localStorage.setItem("space4x_session_key", sessionKey);
      }
      this.credential = sessionKey;
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

  private headers(): Record<string, string> {
    const h: Record<string, string> = {};

    if (this.credential) {
      const isJwt = this.credential.split(".").length === 3;
      if (isJwt) {
        h["Authorization"] = `Bearer ${this.credential}`;
      } else {
        // Send as session key for the custom AuthPlayer extractor
        h["X-Session-Key"] = this.credential;
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
      headers: {
        "Content-Type": "application/json",
        "Accept": "application/json",
        ...this.headers(),
      },
      body: body ? JSON.stringify(body) : undefined,
    });

    if (!res.ok) {
      const text = await res.text();
      throw new Error(text || res.statusText);
    }

    // Defensive check: Ensure we actually received JSON
    const contentType = res.headers.get("content-type");
    if (!contentType || !contentType.includes("application/json")) {
      const htmlText = await res.text();
      throw new Error(
        `Expected JSON from ${BASE + path}, but received HTML/Text. Check your BASE URL or API proxy settings.\nResponse preview: ${htmlText.slice(0, 150)}...`
      );
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
    return this.request("POST", `/units/${unit_id}/move`, {
      to_face,
      to_u,
      to_v,
    });
  }
}

export const api = new ApiClient();
