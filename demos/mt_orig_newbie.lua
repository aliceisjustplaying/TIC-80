-- mt here
-- back to fft shenanigans
-- then maybe twisty donuts

-- greets to alia, aldroidia, ferris
-- gasman, hoffft_maxan, jtruk, and you!

-- alice was here

m=math
sin=m.sin
cos=m.cos
pi=m.pi
fft_current={}
fft_max={}
fft_normalized={}
fft_history={}
fft_smooth={}
-- what is this? arc? all?
fa={}

function BOOT()
 for i=0,255 do
  fft_current[i]=0
  fft_max[i]=0
  fft_normalized[i]=0
  fft_history[i]=0
  fft_smooth[i]=0
  fa[i]=0
 end
end

function BDR(l)
 vbank(0)
 if l == 0 then
 for i=1,15 do
  poke(0x3fc0+i*3, (i*15+t*.3)%255)
  poke(0x3fc0+i*3+1, (i*15+t*.5+60)%255)
  poke(0x3fc0+i*3+2, (i*15+t*.7+120)%255)
 end
 end
end

function TIC()t=time()/100
 cls()
 smooth = 0.7
 for i=0,255 do
  fft_current[i] = fft2(i, true, smooth)*.001
  --fft_current[i] = fft(i)*.1
  if fft_current[i] > fft_max[i] then fft_max[i] = fft_current[i] end
  fft_normalized[i] = fft_current[i]/fft_max[i]
  fa[i] = fa[i] + fft_current[i]
  fft_history[i] = fft_history[i]*.95 + fft_normalized[i]*.05
  bi = m.max(0,i-5)
  ei = m.min(255,i+5)
  fft_smooth[i]=0
  for i = bi,ei do
   fft_smooth[i] = fft_smooth[i]+fft_history[i]*.1
  end
 end

 vbank(0)
 cls()

 for i=0,239 do
  line(i,10,i,10-fft_current[i]*100,2)
  print("fft current * 100",0,12,2,true,1,true)

  line(i,36,i,36-fft_max[i]*100,4)
  print("fft max * 100",0,38,4,true,1,true)

  line(i,62,i,62-fft_normalized[i]*10,6)
  print("fft normalized * 10",0,64,6,true,1,true)

  line(i,88,i,88-fft_history[i]*10,8)
  print("fft history * 10",0,90,8,true,1,true)

  line(i,114,i,114-fft_smooth[i]*10,10)
  print("fft smooth * 10",0,116,10,true,1,true)


 end
 
 vbank(1)
 cls(0)
 
 for i=0,255,.5 do
  cx=160
  cy=68
  r = 50
  a = i/255 * pi * 2 + fa[0]*10
  
  
  mx = r * sin(a)  
  my = r * cos(a)
  w = 5*(1+10*fft_smooth[i//1])
  
  ra = i/255+t/500
  x1= w*sin(ra)
  y1= w*cos(ra)
  x2= w*sin(ra+pi/2)
  y2= w*cos(ra+pi/2)
  x3= w*sin(ra+pi)
  y3= w*cos(ra+pi)
  x4= w*sin(ra+pi/2+3)
  y4= w*cos(ra+pi/2+3)

  segment(x1,y1,x2,y2,r,a,12)
  segment(x2,y2,x3,y3,r,a,13)
  segment(x3,y3,x4,y4,r,a,14)
  segment(x4,y4,x1,y1,r,a,15)
 end
end

function segment(x1,y1,x2,y2,r,a,col)
 ix1= (r-x1) * sin(a)
 iy1= (r+y1) * cos(a)
 ix2= (r-x2) * sin(a)
 iy2= (r+y2) * cos(a)
 line(ix1+cx,iy1+cy,ix2+cx,iy2+cy,col)
end

-- <WAVES>
-- 000:00000000ffffffff00000000ffffffff
-- 001:0123456789abcdeffedcba9876543210
-- 002:0123456789abcdef0123456789abcdef
-- </WAVES>

-- <PALETTE>
-- 000:1a1c2c5d275db13e53ef7d57ffcd75a7f07038b76425717929366f3b5dc941a6f673eff7f4f4f494b0c2566c86333c57
-- </PALETTE>

