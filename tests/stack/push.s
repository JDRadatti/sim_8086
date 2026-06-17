BITS 16
push ax
push bx
push cx
push dx
push sp
push bp
push si
push di
push es
push ss
push ds
push cs
push word [bx]
push word [bx + si]
hlt
