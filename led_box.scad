// ═══════════════════════════════════════════════════════════════════
//  LED MATRIX ENCLOSURE — v3: minimal bezel + snap-fit clips
//
//  Matrix : SeenGreat RGB P3.0 64×64 — 192 × 192 × 15 mm, HUB75, 5V/4A
//           Pixels span the full 192 mm (64 × 3 mm), outer LEDs sit
//           ~0.5 mm from the PCB edge → bezel lip is only 0.5 mm.
//  MCU    : ESP32-S3 DevKit — flat against back panel, lower right,
//           4 standoffs (58 × 22 mm hole pitch). USB-C out right wall.
//  Power  : USB-C female breakout — back panel, centred.
//  Joint  : NO screws — 6 cantilever snap clips (2 top, 2 per side).
//           Bottom is tied by the foot-rail screws.
//  Process: FDM, 0.4 mm nozzle, 3 perimeters. PETG preferred (clips
//           flex); PLA works but snap gently.
//
//  ── Parts ─────────────────────────────────────────────────────────
//   front_piece — face down on plate (bezel lip prints on the bed).
//   back_piece  — back face down. Retention tabs + hooks may want
//                 thin support (small cantilevers).
//   foot_rail   — ×2, no support. Mirror one in slicer.
//
//  ── Assembly ──────────────────────────────────────────────────────
//   1. Slide matrix into front piece from behind (0.5 mm lip stops it).
//   2. ESP32 onto back-piece standoffs (4× BT2×6 thread-forming).
//      USB-C breakout slides into its tray in front of the cutout.
//   3. Route HUB75 + power wires through the shelf cable slot.
//   4. Press back piece on until all 6 clips click. Retention tabs
//      press the matrix against the lip.
//      (To open: press a flat tool into each window, lift the hook.)
//   5. Feet from below: 4× M3×16 into the floor pads, heads hidden
//      in the rubber recesses. Wedge feet tilt the display back
//      FOOT_TILT° (taller edge faces forward).
//
//  ── BOM ───────────────────────────────────────────────────────────
//   4× M3×16 socket-cap — feet     4× BT2×6 thread-forming — ESP32
//   2× rubber strips ~14×29 mm — foot recesses
// ═══════════════════════════════════════════════════════════════════

/* [Matrix] */
MTX_W   = 192;
MTX_D   = 15;
MTX_TOL = 0.5;

/* [Walls] */
WALL      = 3;
BACK_W    = 4;
FRONT_LIP = 0.5;   // bezel overlap — LEDs start ~0.5 mm from PCB edge!

/* [Internal zones] */
WIRE_GAP = 13;
BASE_H   = 30;

/* [Power USB-C breakout — back panel, centred] */
PWR_USBC_W = 9.1;
PWR_USBC_H = 3.1;
// Breakout board tray — MEASURE YOUR BOARD and set these three:
PWR_PCB_W   = 9.2;    // board width
PWR_PCB_L   = 16;    // board length (connector edge to front edge)
PWR_PCB_T   = 1.6;   // board thickness
PWR_SHELL_H = 2.6;   // USB-C receptacle shell height (sits on PCB top)
PWR_RELIEF  = 2.5;   // recess into back wall so the plug seats fully
PWR_CLR     = 0.25;  // side clearance in the tray channel
TRAY_RAIL_W  = 2.5;  // C-rail wall thickness
TRAY_LIP     = 0;  // rail lip overhang onto the PCB top
TRAY_FLOOR_T = 2.5;  // tray floor thickness

/* [ESP32-S3 DevKit — flat against back panel, lower right] */
ESP_W            = 69;
ESP_D            = 25.4;
ESP_PCB_T        = 1.6;
ESP_X_MARGIN     = 1;
ESP_Z_MARGIN     = 2.3;
ESP_HOLE_X_PITCH = 58;     // measured
ESP_HOLE_Z_PITCH = 22;     // measured
STOFF_H          = 4;
STOFF_OD         = 6;
STOFF_ID         = 1.6;    // BT2×6 thread-forming pilot (~0.8 × ⌀2)

/* [ESP32 USB-C slot — right side wall] */
ESP_USBC_LEN     = 22;
ESP_USBC_DEPTH   = 6;
ESP_USBC_Z_INSET = 2;

/* [Snap clips — 2 top, 2 per side] */
CLIP_L      = 20;    // lip length along the wall
CLIP_LIP_T  = 1.4;   // lip thickness (outer part of the 3 mm wall)
CLIP_LIP_D  = 10;    // how far the lip reaches past the split
CLIP_WIN_W  = 7;     // window width (along wall)
CLIP_WIN_H  = 2.5;   // window height (in Y)
CLIP_HOOK_W = 6;     // hook width
CLIP_HOOK_T = 1.2;   // hook proudness off the recessed wall
CLIP_Y      = 6;     // window/hook centre distance behind the split
CLIP_CLEAR  = 0.2;   // fit clearance (increase if snap is too tight)

/* [Cable slot through shelf] */
CABLE_X0 = 55;
CABLE_W  = 50;
CABLE_D  = 18;

/* [Feet — 2 side rails, screwed from below] */
FOOT_W    = 20;
FOOT_H    = 15;    // height at the FRONT edge (tallest point)
FOOT_TILT = 8;     // wedge angle (deg) — display leans back this much
FOOT_EXT  = 25;    // rear reach past the back panel (kickstand — keeps
                   // the tilted box from tipping backward)
FOOT_HY   = [10, 21];
PAD_OD    = 8;
PAD_H     = 2;

/* [Hardware] */
M3_TAP    = 2.5;
M3_CLEAR  = 3.4;
M3_HEAD_D = 6.5;
M3_HEAD_H = 3.0;

// ═══ DERIVED ══════════════════════════════════════════════════════
BOX_W   = MTX_W + 2*MTX_TOL + 2*WALL;            // 199
BOX_D   = WALL + MTX_D + WIRE_GAP + BACK_W;      // 35
BOX_H   = WALL + BASE_H + WALL + MTX_W + WALL;   // 231
INT_W   = MTX_W + 2*MTX_TOL;                     // 193
SHELF_Z = WALL + BASE_H;                         // 33
MTX_Z   = SHELF_Z + WALL;                        // 36
SPLIT_Y = WALL + MTX_D;                          // 18

OPEN_W = MTX_W - 2*FRONT_LIP;                    // 191
OPEN_X = (BOX_W - OPEN_W)/2;
OPEN_Z = MTX_Z + FRONT_LIP;

// ESP32
ESP_X0     = BOX_W - WALL - ESP_X_MARGIN - ESP_W;   // 126
ESP_Z0     = WALL + ESP_Z_MARGIN;                    // 5.3
ESP_BACK_Y = BOX_D - BACK_W - STOFF_H;               // 27
ESP_FACE_Y = ESP_BACK_Y - ESP_PCB_T;                 // 25.4
ESP_HX1 = ESP_X0 + (ESP_W - ESP_HOLE_X_PITCH)/2;
ESP_HX2 = ESP_HX1 + ESP_HOLE_X_PITCH;
ESP_HZ1 = ESP_Z0 + (ESP_D - ESP_HOLE_Z_PITCH)/2;
ESP_HZ2 = ESP_HZ1 + ESP_HOLE_Z_PITCH;

// Power breakout tray (board lies flat, connector poking into the
// back-wall recess; slides in from the front along +Y)
PWR_PCB_ZB = WALL + BASE_H/2 - PWR_SHELL_H/2 - PWR_PCB_T + 2;  // PCB bottom ≈ 15.1
PWR_PCB_Y1 = BOX_D - BACK_W + PWR_RELIEF;                   // rear edge  = 33.5
PWR_PCB_Y0 = PWR_PCB_Y1 - PWR_PCB_L;                        // front edge = 15.5
TRAY_Y0    = PWR_PCB_Y0;
TRAY_X0    = BOX_W/2 - PWR_PCB_W/2 - PWR_CLR - TRAY_RAIL_W;
TRAY_W     = PWR_PCB_W + 2*PWR_CLR + 2*TRAY_RAIL_W;
TRAY_TOP   = PWR_PCB_ZB + PWR_PCB_T + 0.3 + 1.4;            // rail top ≈ 18.4

// Foot screws (X = rail centres); rails run FOOT_EXT past the back
_FOOT_X = [FOOT_W/2, BOX_W - FOOT_W/2];              // 10, 189
FOOT_D  = BOX_D + FOOT_EXT;                          // rail depth = 60

// Clip frames: local (u=along wall, y=box depth, t=outward normal, 0 at
// outer surface). One multmatrix per clip.
// Side clips at z = 100 / 170 (clear of retention tabs at z 66-74 & 196-204)
// Top clips  at x = 45 / 154  (clear of top tabs   at x 60-68 & 131-139)
_CLIPS = [
    [[0,0, 1, BOX_W], [0,1,0,0], [1,0,0, 100], [0,0,0,1]],   // right wall
    [[0,0, 1, BOX_W], [0,1,0,0], [1,0,0, 170], [0,0,0,1]],
    [[0,0,-1, 0    ], [0,1,0,0], [1,0,0, 100], [0,0,0,1]],   // left wall
    [[0,0,-1, 0    ], [0,1,0,0], [1,0,0, 170], [0,0,0,1]],
    [[1,0,0, 45    ], [0,1,0,0], [0,0,1, BOX_H], [0,0,0,1]], // top wall
    [[1,0,0, 154   ], [0,1,0,0], [0,0,1, BOX_H], [0,0,0,1]]
];

// ── Design-rule checks ────────────────────────────────────────────
assert(FRONT_LIP <= 0.6,          "lip covers the outer LED row");
assert(ESP_Z0 + ESP_D <= SHELF_Z, "ESP32 taller than base zone");
assert(WALL + PAD_H <= ESP_Z0,    "floor pads hit ESP32 PCB");
assert(FOOT_HY[0] < SPLIT_Y && FOOT_HY[1] > SPLIT_Y,
       "feet must screw into BOTH halves");
assert(CLIP_LIP_T + CLIP_CLEAR < WALL, "clip recess eats whole wall");
assert(PWR_RELIEF < BACK_W,            "breakout recess breaks through wall");
assert(PWR_PCB_Y0 > SPLIT_Y - 10,      "breakout board too long for chamber");
assert(TRAY_TOP < SHELF_Z,             "tray taller than base zone");
assert(FOOT_H - FOOT_D*tan(FOOT_TILT) > 5,
       "foot tail vanishes — reduce FOOT_TILT or FOOT_EXT");
assert(FOOT_H - FOOT_HY[1]*tan(FOOT_TILT) > 6 + M3_HEAD_H + 3,
       "feet too short at the rear screws for head + threads");
// Tip-over check: rear desk contact must sit behind the tilted CG
// (CG ≈ box centre, ~140 mm up incl. feet, shifted back by the tilt)
assert(FOOT_D*cos(FOOT_TILT) > BOX_D/2 + 140*sin(FOOT_TILT) + 10,
       "kickstand too short for this tilt — increase FOOT_EXT");

echo(str("Box W×D×H : ", BOX_W, " × ", BOX_D, " × ", BOX_H));
echo(str("Bezel opening: ", OPEN_W, " × ", OPEN_W, " (lip ", FRONT_LIP, " mm)"));

// ═══ SNAP CLIP GEOMETRY (local frame) ═════════════════════════════
// Front piece: lip slab with a window.
module _clip_lip_add() {
    difference() {
        translate([-CLIP_L/2, SPLIT_Y, -CLIP_LIP_T])
            cube([CLIP_L, CLIP_LIP_D, CLIP_LIP_T]);
        translate([-CLIP_WIN_W/2, SPLIT_Y + CLIP_Y - CLIP_WIN_H/2, -CLIP_LIP_T - 0.1])
            cube([CLIP_WIN_W, CLIP_WIN_H, CLIP_LIP_T + 0.2]);
    }
}
// Back piece: recess cut (thins the wall to a flexing cantilever).
module _clip_recess_cut() {
    translate([-CLIP_L/2 - CLIP_CLEAR, SPLIT_Y - 0.1, -CLIP_LIP_T - CLIP_CLEAR])
        cube([CLIP_L + 2*CLIP_CLEAR, CLIP_LIP_D + CLIP_CLEAR + 0.1,
              CLIP_LIP_T + CLIP_CLEAR + 1]);
}
// Back piece: ramped hook on the recessed surface. Square face locks
// against the window's rear edge; 45°-ish ramp leads in during snap.
module _clip_hook_add() {
    y0 = SPLIT_Y + CLIP_Y - CLIP_WIN_H/2 + CLIP_CLEAR;   // front face
    t0 = -CLIP_LIP_T - CLIP_CLEAR;                        // recessed surface
    translate([-CLIP_HOOK_W/2, 0, 0])
        rotate([90, 0, 90])
            linear_extrude(CLIP_HOOK_W)
                polygon([[y0, t0], [y0 + 2, t0],
                         [y0 + 2, t0 + CLIP_HOOK_T], [y0 + 1, t0 + CLIP_HOOK_T]]);
}

// ═══ SHARED CUTOUTS ═══════════════════════════════════════════════
module _pockets() {
    translate([WALL, WALL, MTX_Z])                                   // matrix + wire gap
        cube([INT_W, MTX_D + WIRE_GAP, MTX_W]);
    translate([WALL, WALL, WALL])                                    // base chamber
        cube([INT_W, BOX_D - WALL - BACK_W, BASE_H]);
    translate([OPEN_X, -1, OPEN_Z])                                  // bezel opening
        cube([OPEN_W, WALL + 2, OPEN_W]);
    for (i = [0:2]) {                                                // vents
        translate([28 + i*9,         WALL + 5, -1])
            cube([5, BOX_D - 2*WALL - 10, WALL + 2]);
        translate([BOX_W - 33 - i*9, WALL + 5, -1])
            cube([5, BOX_D - 2*WALL - 10, WALL + 2]);
    }
    for (fx = _FOOT_X, fy = FOOT_HY)                                 // foot pilots
        translate([fx, fy, -1])
            cylinder(d=M3_TAP, h=WALL + PAD_H + 7, $fn=16);
    translate([BOX_W - WALL - 1,                                     // ESP32 USB-C slot
               ESP_FACE_Y - ESP_USBC_DEPTH + 5,
               ESP_Z0 + ESP_USBC_Z_INSET])
        cube([WALL + 2, ESP_USBC_DEPTH, ESP_USBC_LEN]);
}

module _cable_slot() {
    translate([CABLE_X0, BOX_D - BACK_W - CABLE_D, SHELF_Z - 1])
        cube([CABLE_W, CABLE_D, WALL + 2]);
}

module _floor_pad(fx, fy) {
    translate([fx, fy, WALL])
        difference() {
            cylinder(d=PAD_OD, h=PAD_H, $fn=24);
            translate([0,0,-1]) cylinder(d=M3_TAP, h=PAD_H+2, $fn=16);
        }
}

// ═══ POWER BREAKOUT MOUNT ═════════════════════════════════════════
// Recess in the back wall's inner face: the board's rear (connector
// end) nests PWR_RELIEF deep so the receptacle reaches the cutout and
// a plug can seat fully (overmold stops at the outer surface).
module _pwr_relief_cut() {
    
}
// Tray on the back piece: floor + two C-rails. Board slides in from
// the front (+Y) under the rail lips until it bottoms in the recess.
module _pwr_tray() {
    // floor
    translate([TRAY_X0, TRAY_Y0, PWR_PCB_ZB - TRAY_FLOOR_T - 0.9])
        cube([TRAY_W, BOX_D - BACK_W - TRAY_Y0, TRAY_FLOOR_T]);
    translate([TRAY_X0, TRAY_Y0, PWR_PCB_ZB + TRAY_FLOOR_T + 0.4])
        cube([TRAY_W, BOX_D - BACK_W - TRAY_Y0, TRAY_FLOOR_T]);
    // rails with lips
    for (rx = [TRAY_X0, TRAY_X0 + TRAY_W - TRAY_RAIL_W]) {
        translate([rx, TRAY_Y0, PWR_PCB_ZB - TRAY_FLOOR_T])
            cube([TRAY_RAIL_W, BOX_D - BACK_W - TRAY_Y0,
                  TRAY_TOP - PWR_PCB_ZB + TRAY_FLOOR_T]);
        if (TRAY_LIP > 0) {
            lip_x = (rx == TRAY_X0) ? rx + TRAY_RAIL_W : rx - TRAY_LIP;
            translate([lip_x, TRAY_Y0, PWR_PCB_ZB + PWR_PCB_T + 0.3])
                cube([TRAY_LIP, BOX_D - BACK_W - TRAY_Y0,
                      TRAY_TOP - PWR_PCB_ZB - PWR_PCB_T - 0.3]);
        }
    }
}
// Blocker fin on the FRONT piece: once the halves clip together it
// sits 0.4 mm in front of the board's edge — the board is captive
// with no fasteners. Open the box to slide the board out.
module _pwr_blocker() {
    translate([BOX_W/2 - 1.1, WALL, WALL])
        cube([2.2, PWR_PCB_Y0 - 0.4 - WALL, 16]);
}

// ═══ FRONT PIECE  (y = 0 … SPLIT_Y, + clip lips) ══════════════════
module front_piece() {
    intersection() {
        difference() { cube([BOX_W, BOX_D, BOX_H]); _pockets(); }
        cube([BOX_W, SPLIT_Y, BOX_H]);
    }
    difference() {                                                   // shelf, front part
        translate([WALL, WALL, SHELF_Z])
            cube([INT_W, SPLIT_Y - WALL, WALL]);
        _cable_slot();
    }
    for (m = _CLIPS) multmatrix(m) _clip_lip_add();                  // clip lips
    _pwr_blocker();                                                  // traps breakout board
    for (fx = _FOOT_X) _floor_pad(fx, FOOT_HY[0]);
}

// ═══ BACK PIECE  (y = SPLIT_Y … BOX_D) ════════════════════════════
module back_piece() {
    difference() {
        union() {
            intersection() {
                difference() { cube([BOX_W, BOX_D, BOX_H]); _pockets(); }
                translate([0, SPLIT_Y, 0])
                    cube([BOX_W, BOX_D - SPLIT_Y, BOX_H]);
            }
            difference() {                                           // shelf, rear part
                translate([WALL, SPLIT_Y, SHELF_Z])
                    cube([INT_W, BOX_D - BACK_W - SPLIT_Y, WALL]);
                _cable_slot();
            }
        }
        translate([BOX_W/2 - PWR_USBC_W/2, BOX_D - BACK_W - 1,       // power USB-C
                   WALL + BASE_H/2 - PWR_USBC_H/2])
            cube([PWR_USBC_W, BACK_W + 2, PWR_USBC_H]);
        _pwr_relief_cut();                                           // breakout nesting recess
        for (m = _CLIPS) multmatrix(m) _clip_recess_cut();           // clip recesses
    }

    _pwr_tray();                                                     // breakout board tray

    for (m = _CLIPS) multmatrix(m) _clip_hook_add();                 // clip hooks

    for (cx = [ESP_HX1, ESP_HX2], cz = [ESP_HZ1, ESP_HZ2])           // ESP32 standoffs
        translate([cx, BOX_D - BACK_W, cz])
            rotate([90, 0, 0])
                difference() {
                    cylinder(d=STOFF_OD, h=STOFF_H, $fn=24);
                    translate([0,0,-2]) cylinder(d=STOFF_ID, h=STOFF_H+3, $fn=16);
                }

    _TAB = 5; _TABL = 6; _TABH = 8;                                  // retention tabs
    for (tz = [MTX_Z + 30, MTX_Z + 160]) {
        translate([WALL, SPLIT_Y, tz])              cube([_TAB, _TABL, _TABH]);
        translate([BOX_W - WALL - _TAB, SPLIT_Y, tz]) cube([_TAB, _TABL, _TABH]);
    }
    for (tx = [60, BOX_W - 68])
        translate([tx, SPLIT_Y, BOX_H - WALL - _TABH])
            cube([_TABH, _TABL, _TABH]);

    for (fx = _FOOT_X) _floor_pad(fx, FOOT_HY[1]);
}

// ═══ FOOT RAIL  (×2 — mirror one in slicer) ═══════════════════════
// Wedge feet: bottom face slopes up toward the rear by FOOT_TILT, so
// on a flat desk the whole box leans back FOOT_TILT degrees.
// The rail runs FOOT_EXT past the back panel as a kickstand — the
// rear desk contact lands well behind the tilted centre of gravity.
module foot_rail() {
    INSET = 3; RDEPTH = 3;
    difference() {
        hull() {
            for (x = [3, FOOT_W-3], y = [3, FOOT_D-3])
                translate([x, y, 0]) cylinder(r=3, h=FOOT_H, $fn=24);
        }

        // Sloped bottom: full FOOT_H at the front (y=0), shorter at
        // the rear — cut everything below the plane z = y·tan(TILT)
        rotate([FOOT_TILT, 0, 0])
            translate([-1, -10, -60])
                cube([FOOT_W + 2, FOOT_D + 20, 60]);

        // Rubber recess, parallel to the sloped bottom
        rotate([FOOT_TILT, 0, 0])
            translate([INSET, INSET, -0.1])
                cube([FOOT_W - 2*INSET, FOOT_D - 2*INSET, RDEPTH + 0.1]);

        // Screw shafts + head counterbores (box-vertical axes; the
        // counterbore floor follows the slope at each hole's y)
        for (fy = FOOT_HY) translate([FOOT_W/2, fy, 0]) {
            translate([0,0,-6]) cylinder(d=M3_CLEAR, h=FOOT_H+8, $fn=20);
            translate([0,0, fy*tan(FOOT_TILT) + RDEPTH - 0.1])
                cylinder(d=M3_HEAD_D, h=M3_HEAD_H+0.2, $fn=24);
        }
    }
}

// ═══ RENDER ═══════════════════════════════════════════════════════
PART          = "foot";   // "front" | "back" | "foot" | "assembly"
CROSS_SECTION = false;

module _render() {
    if (PART == "front") color("LightSteelBlue") front_piece();
    else if (PART == "back")  color("SlateGray") back_piece();
    else if (PART == "foot")  color("SteelBlue") foot_rail();
    else {
        color("LightSteelBlue", 0.9) front_piece();
        color("SlateGray", 0.9)      back_piece();
        color("SteelBlue", 0.9) {
            translate([0, 0, -FOOT_H]) foot_rail();
            translate([BOX_W, 0, -FOOT_H]) mirror([1,0,0]) foot_rail();
        }
    }
}

if (CROSS_SECTION)
    intersection() { _render(); cube([BOX_W, BOX_D/2, BOX_H]); }
else
    _render();
