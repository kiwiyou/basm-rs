; -*- tab-width: 4 -*-
; Copyright (c) 2022, Ilya Kurdyukov
; Copyright (c) 2023, Byeongkeun Ahn
; All rights reserved.
;
; Micro LZMA decoder for x86_64 (static)
;
; This software is distributed under the terms of the
; Creative Commons Attribution 3.0 License (CC-BY 3.0)
; http://creativecommons.org/licenses/by/3.0/
;
; THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
; OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
; FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
; AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
; LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
; OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
; THE SOFTWARE.

; build: nasm -f bin -O9 static-pie-stub-amd64.asm -o static-pie-stub-amd64.bin

BITS 64
ORG 0
section .text

%assign loc_pos 0
%macro LOC 1-3 4, dword
%assign loc_pos loc_pos+%2
%ifidn %3, none
%xdefine %1 [rbp-loc_pos]
%else
%xdefine %1 %3 [rbp-loc_pos]
%endif
%endmacro
LOC _rep0
LOC _rep1
LOC _rep2
LOC _rep3
%assign loc_rep loc_pos
LOC Code, 8
%assign loc_code loc_pos
LOC Range
%assign loc_range loc_pos
LOC _dummyF
LOC _state, 8

%define _rc_bit rdi
; src + 1+4+8+1+4
%define Src r8
%define Dest r9
%define Temp rbp


; Does not touch rdi until we call svc_alloc_rwx
_start:
    push    rdi             ; Save non-volatile registers per win64 calling convention
    push    rsi
    push    rbx
    push    r12
    push    r13
    push    r14
    push    r15
    enter   40, 0           ; shadow space
    push    rdx             ; LZMA binary
    pop     rsi             ; "mov rsi, rdx": rsi is preserved upon function calls
    push    rcx             ; PLATFORM_DATA table
    pop     rbx             ; "mov rbx, rcx": rbx is preserved upon function calls

    xor     eax, eax
    lodsb
    mov     r13, rax                ; r13 = (1 << pb) - 1
    lodsb
    mov     r14, rax                ; r14 = (1 << lp) - 1
    lodsb
    mov     r15, rax                ; r15 = lc
    lodsb
    mov     ecx, eax                ; rcx = lp + lc + 8

    mov     al, 3
    shl     eax, cl
    add     eax, 2048
    xchg    rax, r12                ; r12 = tsize (always a multiple of 256)

    lodsd
    xchg    eax, ecx                ; svc_alloc_rwx: size of memory
    call    qword [rbx + 32]        ; allocate the Dest memory
    push    rax                     ; Save rax = Dst
    xchg    rax, r9                 ; r9 = Dst

    lodsd
    mov     r8, rsi                 ; r8 = Src + 12
    xchg    eax, esi                ; esi = initial 32 bits of the stream
                                    ; Note: the first byte of the LZMA stream is always the zero byte (ignored),
                                    ;       but it is stripped by the packager and does not exist here.
                                    ; Also, the byte swap is also done by the packager.

    push    rbx                     ; Save rbx
    push    rbp                     ; Save rbp
    lea     rdi, [rsp - 2]
    sub     rsp, r12
    sub     rsp, r12

_lzma_dec:
    mov     ecx, r12d
    mov     ax, 1024
    std                     ; set direction flag
    rep     stosw
    cld                     ; clear direction flag
    push    rsp
    pop     rbp
    or      eax, -1         ; 0xffffffff
    add     rax, 2          ; 0x100000001
    push    rax
    push    rax
    push    rsi             ; Code
_rel_code:
    push    -1              ; Range
    push    rcx             ; _state
    ; bh=4, but it doesn't matter
    xchg    ebx, eax        ; Prev = 0
    call    _loop1
_rc_bit1:
    push    rdx
    call    _rc_norm
    movzx   eax, word [Temp+rsi*2]
    mov     edx, Range
    shr     edx, 11
    imul    edx, eax        ; bound
    sub     Range, edx
    sub     Code, edx
    jae    .1
    mov     Range, edx
    add     Code, edx
    cdq
    sub     eax, 2048-31
.1: shr     eax, 5           ; eax >= 0
    sub     [Temp+rsi*2], ax
    neg     edx
    pop     rdx
    ret

_rc_norm:
    cmp     byte [rbp-loc_range+3], 0
    jne     .1
%if 1    ; -2
    shl     qword [rbp-loc_range], 8
%else
    shl     Range, 8
    shl     Code, 8
%endif
    mov     al, [Src]
    inc     Src
    mov     [rbp-loc_code], al
.1: ret

_loop1:
    pop     _rc_bit
_loop:
    mov     rcx, Dest
    mov     al, r14b
    mov     bh, al
_rel_lp:
    pop     rsi             ; _state
    push    rsi
    and     bh, cl
    and     ecx, r13d       ; posState
_rel_pb:
    shl     esi, 5          ; state * 16

    ; probs + state * 16 + posState
    lea     esi, [rsi+rcx*2+64]
    call    _rc_bit
    cdq
    pop     rax
    jc      _case_rep
    mov     ecx, r15d
    shl     ebx, cl
_rel_lc:
    mov     bl, 0
    lea     ecx, [rbx+rbx*2+2048]
_case_lit:
    lea     ebx, [rdx+1]
    ; state = 0x546543210000 >> state * 4 & 15;
    ; state = state < 4 ? 0 : state - (state > 9 ? 6 : 3)
.4: add     al, -3
    sbb     dl, dl
    and     al, dl
    cmp     al, 7
    jae     .4
    push    rax        ; _state
%if 0    ; -2 bytes, but slower
    ; will read one byte before Dest
    add     al, -4
    sbb     bh, bh
%else
    cmp     al, 7-3
    jb      .2
    mov     bh, 1     ; offset
%endif
    mov     eax, _rep0
    neg     rax
    ; dl = -1, dh = 0, bl = 1
    xor     dl, [Dest+rax]
.1: xor     dh, bl
    and     bh, dh
.2: shl     edx, 1
    mov     esi, ebx
    and     esi, edx
    add     esi, ebx
    add     esi, ecx
    call     _rc_bit
    adc     bl, bl
    jnc     .1
    cdq     ; _len
    jmp     _copy.2

_case_rep:
    mov     ebx, esi
    lea     esi, [rdx+rax*4+16]    ; IsRep
    add     al, -7
    sbb     al, al
    and     al, 3
    push    rax        ; _state
    call    _rc_bit
    jc      .2
    ; r3=r2, r2=r1, r1=r0
%if 1
    movups  xmm0, [rbp-loc_rep]
    movups  [rbp-loc_rep-4], xmm0
%else
    mov     rsi, [rbp-loc_rep+8]
    xchg    rsi, [rbp-loc_rep+4]
    mov     _rep3, esi
%endif
    ; state = state < 7 ? 0 : 3
    mov     dl, 819/9    ; LenCoder
    jmp     _case_len

.2: inc     esi
    call    _rc_bit
    jc      .3
    lea     esi, [rbx+1]    ; IsRep0Long
    call    _rc_bit
    jc      .5
    ; state = state < 7 ? 9 : 11
    or      _state, 9
    ; edx = 0, _len
    jmp     _copy.1

.3: mov     dl, 3
    mov     ebx, _rep0
.6: inc     esi
    dec     edx
    xchg    [rbp-loc_rep+rdx*4], ebx
    je      .4
    call    _rc_bit
    jc      .6
.4: mov     _rep0, ebx
.5: ; state = state < 7 ? 8 : 11
    or      _state, 8
    mov     dl, 1332/9    ; RepLenCoder
_case_len:
    lea     esi, [rdx*8+rdx]
    cdq
    call    _rc_bit
    inc     esi
    lea     ebx, [rsi+rcx*8]    ; +1 unnecessary
    mov     cl, 3
    jnc     .4
    mov     dl, 8/8
    call    _rc_bit
    jnc     .3
    ; the first byte of BitTree tables is not used,
    ; so it's safe to add 255 instead of 256 here
    lea     ebx, [rsi+127]
    mov     cl, 8
    add     edx, 16/8-(1<<8)/8    ; edx = -29
.3: sub     ebx, -128    ; +128
.4:     ; BitTree
    push    1
    pop     rsi
    push    rsi
.5: push    rsi
    add     esi, ebx
    call    _rc_bit
    pop     rsi
    adc     esi, esi
    loop    .5
    lea     ebx, [rsi+rdx*8+2-8-1]
    cmp     _state, 4
    pop     rdx    ; edx = 1
    push    rbx    ; _len
    jae     _copy
_case_dist:
    add     _state, 7
    sub     ebx, 3+2-1
    sbb     eax, eax
    and     ebx, eax
    lea     ebx, [rdx-1+rbx*8+(432+16-128)/8+(3+2)*8]    ; PosSlot
    ; BitTree
    push    rdx
.5: lea     esi, [rdx+rbx*8]
    call    _rc_bit
    adc     edx, edx
    mov     ecx, edx
    sub     ecx, 1<<6
    jb      .5
    pop     rbx    ; ebx = 1
_case_model:
    cmp     ecx, 4
    jb      .9
    mov     esi, ebx
    shr     ecx, 1
    rcl     ebx, cl
    dec     ecx
    not     dl    ; 256-edx-1
    mov     dh, 2
    add     edx, ebx
;   lea     edx, [rdx+rbx+688+16+64-256*3]    ; SpecPos
    cmp     ecx, 6
    jb      .4
.1: dec     ecx
    call    _rc_norm
    shr     Range, 1
    mov     edx, Range
    cmp     Code, edx
    jb      .3
    sub     Code, edx
    bts     ebx, ecx
.3: cmp     ecx, 4
    jne     .1
    cdq        ; Align
.4:
.5: push    rsi
    add     esi, edx
    call    _rc_bit
    pop     rsi
    adc     esi, esi
    loop    .5
.6: adc     ecx, ecx
    shr     esi, 1
    jne     .6
    add     ecx, ebx
.9: inc     ecx
    mov     _rep0, ecx
    je      _end
_copy:
    pop     rdx
.1: mov     ecx, _rep0
    neg     rcx
    movzx   ebx, byte [Dest+rcx]
.2: mov     [Dest], bl    ; Dict + Pos
    inc     Dest
    dec     edx
    jns     .1
    jmp     _loop
_end:
_code_end:
    lea     rsp, [rsp + 2*r12 + 48] ; Restore rsp
    pop     rbp                     ; Restore rbp
    pop     rcx                     ; rcx = PLATFORM_DATA table
    pop     rax                     ; rax = start of the binary
    add     rax, qword [Dest - 8]   ; add entrypoint offset
    call    rax                     ; Jump to the entrypoint of the binary
                                    ; (it will inherit the current stackframe)
    leave
    pop     r15                     ; Restore non-volatile registers
    pop     r14
    pop     r13
    pop     r12
    pop     rbx
    pop     rsi
    pop     rdi
    ret