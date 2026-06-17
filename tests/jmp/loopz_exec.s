BITS 16
mov ax, 1
mov cx, 3
label:
sub ax, 1
loopz label
