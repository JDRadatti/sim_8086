BITS 16
mov ax, [bx+si]
mov [bx+di], cx
mov dx, [bp+si]
mov [bp+di], ax
mov [si], bl
mov bx, [di]
hlt
