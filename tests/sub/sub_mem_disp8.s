BITS 16
sub [bx+2], ax
sub al, [bx+si+4]
sub cx, [bx+di+127]
sub dl, [bp+si+1]
sub [bp+di+0], al
sub [si+64], bl

hlt