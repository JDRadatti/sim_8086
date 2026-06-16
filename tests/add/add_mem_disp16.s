BITS 16
add ax, [bx+si+256]
add [bx+di+1024], cx
add dl, [bp+si+512]
add [bp+di+8192], al
add [si+16384], bl
add bx, [di+4096]
add [bp+2048], al
add dh, [bx+32767]

hlt
