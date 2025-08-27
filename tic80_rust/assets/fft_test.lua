-- Simple FFT visualization test cart
-- Usage:
--   cargo run --manifest-path tic80_rust/Cargo.toml -- tic80_rust/assets/fft_test.lua --audio-device "..."

function BOOT()
  cls(0)
end

function TIC()
  cls(0)
  local bars = 32
  local sw, sh = 240, 136
  local bw = math.floor(sw / bars)
  for i=0,bars-1 do
    local v = fft(i, -1) -- normalized bin value
    if v < 0 then v = 0 end
    if v > 1 then v = 1 end
    local h = math.floor(v * (sh-10))
    local x = i * bw
    local y = sh - h
    rect(x, y, bw-1, h, 6)
  end
  print("fft test (normalized)", 2, 2, 14, false, 1, false)
end

