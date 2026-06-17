BITS 16
mov ax, 0x1234
push ax
mov bx, 0x5678
push bx
pop ax
pop bx
hlt
