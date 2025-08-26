-- A calmer demo showcasing: cls, pix, line, rect, rectb, clip, print
local t = 0
local noise = {}
local seed = 123456789 -- deterministic noise

local function lcg()
  seed = (seed * 1664525 + 1013904223) % 4294967296
  return seed
end

function BOOT()
  cls(1) -- deep blue background
  -- Precompute a bunch of random pixel positions to avoid flicker
  for i = 1, 400 do
    local x = lcg() % 240
    local y = lcg() % 136
    noise[i] = { x, y }
  end
end

function TIC()
  -- Stable background each frame to avoid trails
  cls(1)

  -- Crosshair via pix at the center
  local cx, cy = 120, 68
  for dx = -10, 10 do pix(cx + dx, cy, 7) end
  for dy = -10, 10 do pix(cx, cy + dy, 7) end

  -- Title and legend
  print("TIC-80 Rust Demo", 8, 6, 15)
  print("cls pix line rect rectb clip print", 8, 16, 14)

  -- Static card: rect + rectb
  rectb(20, 28, 48, 22, 12) -- border
  rect(22, 30, 44, 18, 6)   -- fill

  -- Gentle animation: small square drifting horizontally
  local ax = 90 + (t % 60) - 30
  rectb(ax, 34, 14, 14, 12)
  rect(ax + 1, 35, 12, 12, 3)

  -- Clip demo toggles every ~2.5 seconds (150 frames @60 FPS)
  local clipped = (t // 150) % 2 == 0
  if clipped then
    clip(100, 48, 60, 36)
    rect(92, 40, 80, 56, 4) -- clipped fill
    clip()
    rectb(100, 48, 60, 36, 14)
    print("clip: ON", 102, 40, 14)
  else
    clip()
    rect(92, 40, 80, 56, 2) -- un-clipped fill (subtle change)
    rectb(100, 48, 60, 36, 14)
    print("clip: OFF", 102, 40, 14)
  end

  -- Subtle diagonal lines
  line(0, 0, 239, 135, 13)
  line(0, 135, 239, 0, 2)

  -- Sprinkle precomputed random pixels ("stars")
  for i = 1, #noise do
    local p = noise[i]
    pix(p[1], p[2], 13)
  end

  t = t + 1
end
