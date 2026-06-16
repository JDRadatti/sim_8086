BITS 16
mov ax, 0x1234
mov [0x0100], ax
mov bx, [0x0100]
hlt
