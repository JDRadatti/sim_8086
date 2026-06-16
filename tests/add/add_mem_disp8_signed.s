BITS 16
add al, [bx+si-1]
add cx, [bx+di-128]
add dl, [bp+si-64]
add [bp+di-2], al
add [si-33], bl
add bx, [di-17]
add [bp-85], al
add dh, [bx-127]

hlt
