BITS 16
mov al, [bx+si-1]
mov cx, [bx+di-128]
mov dl, [bp+si-64]
mov [bp+di-2], ax
mov [si-33], bl
mov bx, [di-17]
mov [bp-85], al
mov dh, [bx-127]

hlt
