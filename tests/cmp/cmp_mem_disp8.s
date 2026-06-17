BITS 16
cmp [bx+2], ax
cmp al, [bx+si+4]
cmp cx, [bx+di+127]
cmp dl, [bp+si+1]
cmp [bp+di+0], al
cmp [si+64], bl

hlt
