BITS 16
mov al, 0
cmp al, 1
jl target
mov al, 0x42
target:
