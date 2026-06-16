BITS 16
add al, [bx+si+4]
add cx, [bx+di+127]
add dl, [bp+si+1]
add [bp+di+0], al
add [si+64], bl
add bx, [di+32]
add [bp+8], al
add dh, [bx+63]

hlt
