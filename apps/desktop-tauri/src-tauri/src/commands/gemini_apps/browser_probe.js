// Runs only inside the selected, persistent Gemini browser profile. Cookies,
// CSRF and page HTML stay in that browser; the native callback gets quota only.
(() => {
  const key = __CODEXBAR_REQUEST_KEY__;
  const controller = new AbortController();
  const state = { kind: "pending", abort: () => controller.abort() };
  window[key] = state;
  const timer = setTimeout(() => controller.abort(), 15000);
  const read = async (response, limit) => {
    if (!response.ok) throw { kind: response.status === 401 || response.status === 403 ? "auth" : "http" };
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let size = 0, text = "";
    try {
      for (;;) {
        const { value, done } = await reader.read();
        if (done) break;
        size += value.byteLength;
        if (size > limit) throw { kind: "too-large" };
        text += decoder.decode(value, { stream: true });
      }
      return text + decoder.decode();
    } finally { await reader.cancel().catch(() => {}); }
  };
  (async () => {
    try {
      if (location.origin !== "https://gemini.google.com") throw { kind: "auth" };
      const options = { credentials: "same-origin", redirect: "error", cache: "no-store", signal: controller.signal };
      const html = await read(await fetch("/app", options), 8 * 1024 * 1024);
      const match = html.match(/"SNlM0e"\s*:\s*("(?:\\.|[^"\\])*")/);
      if (!match) throw { kind: "auth" };
      const csrf = JSON.parse(match[1]);
      if (typeof csrf !== "string" || !csrf) throw { kind: "auth" };
      const response = await fetch("/_/BardChatUi/data/batchexecute?rpcids=jSf9Qc&source-path=%2Fapp&rt=c", {
        ...options, method: "POST", headers: { "X-Same-Domain": "1" },
        body: new URLSearchParams({ "f.req": '[[["jSf9Qc","[]",null,"generic"]]]', at: csrf }),
      });
      const body = await read(response, 128 * 1024);
      if (location.origin !== "https://gemini.google.com") throw { kind: "auth" };
      state.kind = "ok";
      state.body = body;
    } catch (error) {
      state.kind = ["auth", "http", "too-large"].includes(error?.kind) ? error.kind : "network";
    } finally {
      clearTimeout(timer);
      if (window[key] === state) {
        const { abort, ...reply } = state;
        location.assign("https://gemini.google.com/__codexbar_usage_result#request="
          + encodeURIComponent(key) + "&reply=" + encodeURIComponent(JSON.stringify(reply)));
      }
    }
  })();
  return null;
})();
