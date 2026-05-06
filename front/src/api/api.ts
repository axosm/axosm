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
  planet_id: number;
  galaxy_id: number;
  system_id: number;
  planet_seed: number;
  subdivision: number;
  units: Unit[];
  visible_tiles: PlanetTile[];
}

export interface InitResponse {
  state: GameState;
  is_new_player: boolean;
}

// ── API client ────────────────────────────────────────────────

const BASE = '/api';

export class ApiClient {
  private token: string | null = null;

  setToken(t: string) {
    this.token = t;
    localStorage.setItem('space4x_token', t);
  }

  loadToken() {
    this.token = localStorage.getItem('space4x_token');
    return this.token;
  }

  clearToken() {
    this.token = null;
    localStorage.removeItem('space4x_token');
  }

  private headers(): HeadersInit {
    const h: Record<string, string> = { 'Content-Type': 'application/json' };
    if (this.token) h['Authorization'] = `Bearer ${this.token}`;
    return h;
  }

  private async request<T>(method: string, path: string, body?: unknown): Promise<T> {
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

  async register(username: string, email: string, password: string): Promise<AuthResponse> {
    return this.request('POST', '/auth/register', { username, email, password });
  }

  async login(email: string, password: string): Promise<AuthResponse> {
    return this.request('POST', '/auth/login', { email, password });
  }

  async initGame(): Promise<InitResponse> {
    return this.request('POST', '/game/init');
  }

  async getGameState(): Promise<GameState> {
    return this.request('GET', '/game/state');
  }

  async getVisibleTiles(planet_id: number): Promise<PlanetTile[]> {
    return this.request('GET', `/game/visible-tiles?planet_id=${planet_id}`);
  }

  async moveUnit(unit_id: number, to_face: number, to_u: number, to_v: number) {
    return this.request('POST', `/units/${unit_id}/move`, { to_face, to_u, to_v });
  }
}

export const api = new ApiClient();
