// Helper to generate a fast UUID v4 equivalent if crypto.randomUUID isn't available over HTTP
function generateUUID(): string {
  if (typeof crypto !== "undefined" && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

/**
 * Retrieves the existing session key or establishes a brand new unique one.
 */
function getOrCreateSessionKey(): string {
  let sessionKey = localStorage.getItem("game_session_key");

  if (!sessionKey) {
    sessionKey = generateUUID();
    localStorage.setItem("game_session_key", sessionKey);
    console.log("New local player profile initialized:", sessionKey);
  }

  return sessionKey;
}
