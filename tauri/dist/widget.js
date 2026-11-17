// The widget's approved look, in canvas 2D. Shared by the reference page (reference/index.html)
// and the tauri-canvas variant; every other renderer is scored against PNGs exported from the
// reference page. Classic script (not a module) so the reference page also works from file://.
// Needs space-mono.js loaded first; draw only after `Widget.ready` resolves (the font is loaded).
window.Widget = (() => {
  const W = 240, H = 110, R = 20, PIE_Y = 50, ACC = "rgb(80,255,120)";
  const FREEZE = 2; // seconds; the bench sets BENCH_FREEZE to this when it takes screenshots
  const FONT = "12px 'Space Mono'";
  const MARGIN = 8; // room around the panel for its outer glow; the reference PNG includes it
  const font = new FontFace("Space Mono", `url(${window.SPACE_MONO})`);
  document.fonts.add(font);
  const ready = font.load();
  const pieX = (i) => 32 + i * 52;
  const pwm = (t, i) => 0.5 + 0.45 * Math.sin(t * 1.3 + i); // fake PWM

  // Panel, grid, inner glow, border, title and pie rings, at (0, 0) in CSS pixels. These never
  // change, so callers may draw them once into a cache.
  function drawStatic(g) {
    g.save();
    g.save(); // tiny outer glow, drawn under the panel
    g.shadowColor = "rgba(80,255,120,0.45)"; g.shadowBlur = 6;
    g.strokeStyle = "rgba(80,255,120,0.35)";
    g.beginPath(); g.roundRect(0.5, 0.5, W - 1, H - 1, 4); g.stroke();
    g.restore();
    g.beginPath(); g.roundRect(0.5, 0.5, W - 1, H - 1, 4);
    g.fillStyle = "rgba(3,10,3,0.25)"; g.fill();
    g.save(); g.clip();
    g.strokeStyle = "rgba(80,255,120,0.06)"; g.lineWidth = 1;
    for (let y = 0; y < H; y += 12) { g.beginPath(); g.moveTo(0, y + 0.5); g.lineTo(W, y + 0.5); g.stroke(); }
    for (let x = 0; x < W; x += 12) { g.beginPath(); g.moveTo(x + 0.5, 0); g.lineTo(x + 0.5, H); g.stroke(); }
    g.shadowColor = "rgba(80,255,120,0.5)"; g.shadowBlur = 24; // inset glow
    g.strokeStyle = "rgba(80,255,120,0.15)"; g.lineWidth = 2;
    g.beginPath(); g.roundRect(-1, -1, W + 2, H + 2, 4); g.stroke();
    g.restore();
    g.strokeStyle = "rgba(80,255,120,0.35)";
    g.beginPath(); g.roundRect(0.5, 0.5, W - 1, H - 1, 4); g.stroke();
    g.font = FONT; g.fillStyle = ACC; g.shadowColor = ACC; g.shadowBlur = 6;
    g.fillText("MOTORS", 10, 20);
    g.shadowBlur = 0; g.strokeStyle = "rgba(80,255,120,0.5)";
    for (let i = 0; i < 4; i++) { g.beginPath(); g.arc(pieX(i), PIE_Y, R, 0, 2 * Math.PI); g.stroke(); }
    g.restore();
  }

  // Wedges and numbers of the panel whose top-left corner is (x, y), at time t in seconds.
  function drawDynamic(ctx, x, y, t) {
    ctx.save();
    ctx.font = FONT; ctx.textAlign = "center";
    for (let i = 0; i < 4; i++) {
      const f = pwm(t, i);
      ctx.beginPath(); ctx.moveTo(x + pieX(i), y + PIE_Y);
      ctx.arc(x + pieX(i), y + PIE_Y, R, -Math.PI / 2, -Math.PI / 2 + f * 2 * Math.PI);
      ctx.fillStyle = "rgba(80,255,120,0.6)"; ctx.shadowBlur = 0; ctx.fill();
      ctx.fillStyle = ACC; ctx.shadowColor = "rgba(80,255,120,0.6)"; ctx.shadowBlur = 8;
      ctx.fillText(Math.round(1000 + f * 1000), x + pieX(i), y + 94);
    }
    ctx.restore();
  }

  function draw(ctx, x, y, t) {
    ctx.save(); ctx.translate(x, y); drawStatic(ctx); ctx.restore();
    drawDynamic(ctx, x, y, t);
  }

  // Where panel p sits in an overlay, in CSS pixels; the same grid in every variant.
  const origin = (p) => [40 + 260 * (p % 4), 60 + 140 * Math.floor(p / 4)];

  return { W, H, MARGIN, FREEZE, ready, origin, drawStatic, drawDynamic, draw };
})();
