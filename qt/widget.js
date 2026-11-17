.pragma library
// tauri/dist/widget.js (the reference look) ported to Qt Quick's Canvas, which has the same 2D
// API: only roundRect is spelled roundedRect here. Keep the two in step.
var W = 240, H = 110, R = 20, PIE_Y = 50, ACC = "rgb(80,255,120)";
var FONT = "12px 'Space Mono'";
// Qt's shadow blur is wider than a browser's for the same shadowBlur; 0.4 scored closest to the
// reference (8.5% look error, Linux software renderer at scale 2)
var BLUR = 0.4;
var NONE = "rgba(0,0,0,0)"; // Qt ignores shadowBlur = 0, so a transparent shadow color turns it off

function pieX(i) { return 32 + i * 52; }
function pwm(t, i) { return 0.5 + 0.45 * Math.sin(t * 1.3 + i); } // fake PWM
function rounded(g, x, y, w, h, r) { g.beginPath(); g.roundedRect(x, y, w, h, r, r); }

// Panel, grid, glows, border, title and pie rings at (0, 0): painted once.
function drawStatic(g) {
    g.save();
    g.save(); // tiny outer glow, drawn under the panel
    g.shadowColor = "rgba(80,255,120,0.45)"; g.shadowBlur = 6 * BLUR;
    g.strokeStyle = "rgba(80,255,120,0.35)"; g.lineWidth = 1;
    rounded(g, 0.5, 0.5, W - 1, H - 1, 4); g.stroke();
    g.restore();
    rounded(g, 0.5, 0.5, W - 1, H - 1, 4);
    g.fillStyle = "rgba(3,10,3,0.25)"; g.fill();
    g.save(); g.clip();
    g.strokeStyle = "rgba(80,255,120,0.06)"; g.lineWidth = 1;
    for (var y = 0; y < H; y += 12) { g.beginPath(); g.moveTo(0, y + 0.5); g.lineTo(W, y + 0.5); g.stroke(); }
    for (var x = 0; x < W; x += 12) { g.beginPath(); g.moveTo(x + 0.5, 0); g.lineTo(x + 0.5, H); g.stroke(); }
    g.shadowColor = "rgba(80,255,120,0.5)"; g.shadowBlur = 24 * BLUR; // inset glow
    g.strokeStyle = "rgba(80,255,120,0.15)"; g.lineWidth = 2;
    rounded(g, -1, -1, W + 2, H + 2, 4); g.stroke();
    g.restore();
    g.strokeStyle = "rgba(80,255,120,0.35)"; g.lineWidth = 1;
    rounded(g, 0.5, 0.5, W - 1, H - 1, 4); g.stroke();
    g.font = FONT; g.fillStyle = ACC; g.shadowColor = ACC; g.shadowBlur = 6 * BLUR;
    g.fillText("MOTORS", 10, 20);
    g.shadowColor = NONE; g.shadowBlur = 0; g.strokeStyle = "rgba(80,255,120,0.5)";
    for (var i = 0; i < 4; i++) { g.beginPath(); g.arc(pieX(i), PIE_Y, R, 0, 2 * Math.PI); g.stroke(); }
    g.restore();
}

// Wedges and numbers at time t: painted every frame.
function drawDynamic(g, t) {
    g.save();
    g.font = FONT; g.textAlign = "center";
    for (var i = 0; i < 4; i++) {
        var f = pwm(t, i);
        g.beginPath(); g.moveTo(pieX(i), PIE_Y);
        g.arc(pieX(i), PIE_Y, R, -Math.PI / 2, -Math.PI / 2 + f * 2 * Math.PI);
        g.fillStyle = "rgba(80,255,120,0.6)"; g.shadowColor = NONE; g.shadowBlur = 0; g.fill();
        g.fillStyle = ACC; g.shadowColor = "rgba(80,255,120,0.6)"; g.shadowBlur = 8 * BLUR;
        g.fillText(Math.round(1000 + f * 1000), pieX(i), 94);
    }
    g.restore();
}
