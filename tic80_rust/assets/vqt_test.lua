-- VQT visualization cart: 120 bins (12 octaves), 2px per bin across 240px
-- Auto-toggles between unwhitened (vqt/vqts) and whitened (vqtw/vqtsw) every few seconds.

local sw, sh = 240, 136
local bins = 120
local bw = 2          -- 2px per bin -> 120 * 2 = 240
local top = 12        -- top margin for labels

function BOOT()
  cls(0)
end

local function octave_color(bin)
  -- Map octave (0..11) to palette indices with a pleasant ramp
  local oct = math.floor(bin / 12)
  local palette = {6, 7, 10, 12, 14, 9, 11, 15}
  return palette[(oct % #palette) + 1]
end

function TIC()
  cls(0)

  -- Toggle whitened view every 3 seconds
  local ms = time()
  local whiten = ((ms // 3000) % 2) == 1

  -- Draw bars
  local max_v = -1
  local max_bin = 0
  for i = 0, bins - 1 do
    local v = whiten and vqtsw(i) or vqts(i)  -- smoothed normalized
    if v < 0 then v = 0 end
    if v > 1 then v = 1 end
    if v > max_v then max_v = v; max_bin = i end
    local h = math.floor(v * (sh - (top + 8)))
    local x = i * bw
    local y = sh - h
    local c = octave_color(i)
    rect(x, y, bw - 1, h, c)
  end

  -- Highlight peak bin
  rectb(max_bin * bw, sh - math.floor(max_v * (sh - (top + 8))), bw - 1, math.floor(max_v * (sh - (top + 8))), 2)

  -- Draw octave grid lines every 12 bins
  for o = 0, 11 do
    local x = o * 12 * bw
    rect(x, top, 1, sh - (top + 1), 5)
  end

  -- Labels
  local mode = whiten and "whitened" or "raw"
  print("VQT (" .. mode .. ")", 2, 2, 14)
  print("bins: 0..119 (12 octaves)", 2, 6, 13)
end

