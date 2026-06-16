BITS 16
add ax, [bx+si]
add [bx+di], cx
add dx, [bp+si]
add [bp+di], ax
add [si], bl
add bx, [di]

hlt
