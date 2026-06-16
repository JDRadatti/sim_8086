BITS 16
mov bx, 0x0100
mov ax, 0x1234
mov [bx+2], ax
mov ax, [bx+2]
hlt
