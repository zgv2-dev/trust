// The panel in Slint markup, matching the reference (tauri/dist/widget.js) step by step: absolute
// positions, Space Mono, and halo's pre-blurred glow images for the shadows Slint can't draw.
slint::slint! {
    import "../fonts/SpaceMono-Regular.ttf";

    // Glow images rendered by halo at the window's scale; set from Rust once the scale is known.
    export global Glows {
        in property <image> outer;
        in property <image> inner;
        in property <image> title;
        // one property per digit: an image array ([image]) set from Rust doesn't render in Slint 1.18
        in property <image> d0; in property <image> d1; in property <image> d2; in property <image> d3;
        in property <image> d4; in property <image> d5; in property <image> d6; in property <image> d7;
        in property <image> d8; in property <image> d9;
        public pure function digit(d: int) -> image {
            d == 0 ? d0 : d == 1 ? d1 : d == 2 ? d2 : d == 3 ? d3 : d == 4 ? d4 :
            d == 5 ? d5 : d == 6 ? d6 : d == 7 ? d7 : d == 8 ? d8 : d9
        }
    }

    // An image drawn 1:1 in physical pixels, its top-left at (x, y) in logical px.
    component Sprite inherits Image {
        width: self.source.width * 1phx;
        height: self.source.height * 1phx;
    }

    component Wedge inherits Path {
        in property <float> amount; // not `fill`: Path has its own fill brush
        width: 44px; height: 44px;
        viewbox-width: 44; viewbox-height: 44;
        fill: rgba(80, 255, 120, 0.6);
        MoveTo { x: 22; y: 22; }
        LineTo { x: 22; y: 2; }
        ArcTo {
            radius-x: 20; radius-y: 20; large-arc: amount > 0.5; sweep: true;
            x: 22 + 20 * sin(amount * 360deg); y: 22 - 20 * cos(amount * 360deg);
        }
        Close {}
    }

    // Constants from halo: Space Mono 12 px ascent 13.44, advance 7.344.
    component Panel inherits Rectangle {
        width: 240px; height: 110px;
        in property <float> freeze: -1; // >= 0: draw that instant forever, for screenshots
        property <float> t: freeze >= 0 ? freeze : animation-tick() / 1s;
        Sprite { source: Glows.outer; x: -10px; y: -10px; }
        Rectangle { border-width: 1px; border-radius: 4.5px; border-color: rgba(80, 255, 120, 0.35); }
        Rectangle { x: 0.5px; y: 0.5px; width: 239px; height: 109px; border-radius: 4px; background: rgba(3, 10, 3, 0.25); }
        Rectangle {
            x: 0.5px; y: 0.5px; width: 239px; height: 109px; border-radius: 4px; clip: true;
            for k in 10: Rectangle { x: -0.5px; y: k * 12px - 0.5px; width: 240px; height: 1px; background: rgba(80, 255, 120, 0.06); }
            for k in 20: Rectangle { x: k * 12px - 0.5px; y: -0.5px; width: 1px; height: 110px; background: rgba(80, 255, 120, 0.06); }
        }
        Sprite { source: Glows.inner; x: 0; y: 0; }
        Rectangle { border-width: 1px; border-radius: 4.5px; border-color: rgba(80, 255, 120, 0.35); }
        Sprite { source: Glows.title; x: 0; y: 0; } // explicit: Slint centers unpositioned elements
        Text { x: 10px; y: 20px - 13.44px; text: "MOTORS"; color: #50ff78; font-family: "Space Mono"; font-size: 12px; }
        for i in 4: Rectangle {
            x: 32px + i * 52px - 20.5px; y: 50px - 20.5px; width: 41px; height: 41px;
            border-width: 1px; border-radius: 20.5px; border-color: rgba(80, 255, 120, 0.5);
        }
        for i in 4: Rectangle {
            x: 0; y: 0; width: 240px; height: 110px; // explicit: unsized children of a non-layout get centered
            property <float> f: 0.5 + 0.45 * sin((t * 1.3 + i) * 1rad); // fake PWM
            property <int> n: round(1000 + f * 1000);
            property <length> left: 32px + i * 52px - 2 * 7.344px; // 4 digits centered on the pie
            Wedge { x: 32px + i * 52px - 22px; y: 50px - 22px; amount: f; }
            // each digit's glow, pen at (left + k advances, baseline 94); halo sprites start at (-14, -28)
            Sprite { source: Glows.digit(floor(n / 1000)); x: left - 14px; y: 66px; }
            Sprite { source: Glows.digit(mod(floor(n / 100), 10)); x: left + 7.344px - 14px; y: 66px; }
            Sprite { source: Glows.digit(mod(floor(n / 10), 10)); x: left + 2 * 7.344px - 14px; y: 66px; }
            Sprite { source: Glows.digit(mod(n, 10)); x: left + 3 * 7.344px - 14px; y: 66px; }
            Text { x: left; y: 94px - 13.44px; text: n; color: #50ff78; font-family: "Space Mono"; font-size: 12px; }
        }
    }

    // One panel per window, with 8 px around it for the outer glow.
    export component Widget inherits Window {
        no-frame: true;
        always-on-top: true;
        background: transparent;
        width: 256px; height: 126px;
        in property <float> freeze: -1;
        Panel { x: 8px; y: 8px; freeze: root.freeze; }
    }

    export component Overlay inherits Window {
        in property <int> count;
        in property <float> freeze: -1;
        // explicit size: without it the window is pinned to its content's preferred size and
        // macOS ignores any later resize (the overlay stayed small and centered, clipping panels)
        in property <length> w: 240px;
        in property <length> h: 110px;
        width: w; height: h;
        no-frame: true;
        always-on-top: true;
        background: transparent;
        for i in count: Panel {
            freeze: root.freeze;
            x: (40 + 260 * mod(i, 4)) * 1px;
            y: (60 + 140 * floor(i / 4)) * 1px;
        }
    }
}
