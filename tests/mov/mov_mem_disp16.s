BITS 16
mov ax, [bx+si+256]
mov [bx+di+1024], cx
mov dl, [bp+si+512]
mov [bp+di+8192], ax
mov [si+16384], bl
mov bx, [di+4096]
mov [bp+2048], al
mov dh, [bx+32767]

hlt
