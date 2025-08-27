-- Simple time/trace test cart
-- Shows elapsed ms since start and emits a trace every second.

local last_ms = 0
local last_sec = -1

function BOOT()
  cls(0)
end

function TIC()
  cls(0)
  local ms = math.floor(time())
  local sec = math.floor(ms / 1000)

  -- Draw elapsed ms and a ticking marker
  print("time(ms): " .. ms, 2, 2, 14, false, 1, false)
  if (sec % 2) == 0 then
    rect(2, 14, 10, 10, 6)
  else
    rect(2, 14, 10, 10, 12)
  end

  -- Emit a trace once per new second
  if sec ~= last_sec then
    trace("sec=" .. tostring(sec), 7)
    last_sec = sec
  end

  last_ms = ms
end

