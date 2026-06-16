BITS 16
mov al, [bx+si+4]
mov cx, [bx+di+127]
mov dl, [bp+si+1]
mov [bp+di+0], ax
mov [si+64], bl
mov bx, [di+32]
mov [bp+8], al
mov dh, [bx+63]

hlt
