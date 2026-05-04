;Annotated Binary32 Normal Case Subtraction Instruction Trace
;Main Subtraction Fuction Entry Point
[001]sub rsp, 0x58                 ;Allocate 88 bytes on the stack
[002]mov qword ptr [rsp 0x8], rdi  ;Store first parameter (pointer to first Binary32) at rsp+8
[003]mov al, dl                    ;Move the 3rd parameter (lower byte) to al register
[004]mov dword ptr [rsp 0x18], esi ;Store second parameter (second Binary32) at rsp+24
[005]mov ecx, dword ptr [rsp 0x18] ;Load second Binary32 value into ecx
[006]mov dword ptr [rsp 0x14], ecx ;Store second Binary32 value at rsp+20
[007]mov byte ptr [rsp 0x1f], al   ;Store lower byte of 3rd parameter at rsp+31
[008]mov qword ptr [rsp 0x30], rdi ;Store first parameter (pointer to first Binary32) at rsp+48
[009]lea rax, ptr [rip 0x3d6e9]    ;Load effective address relative to program counter into rax
[010]lea rdi, ptr [rsp 0x1f]       ;Load address of rsp+31 into rdi (parameter for next function)
[011]call rax                      ;Call function at address in rax

;Set Rounding Mode
[012]push rax                 ;Save rax register
[013]mov qword ptr [rsp], rdi ;Store rdi at top of stack
[014]call 0x558bdf671e20      ;Call function to set rounding mode

;Convert Rounding Mode to Float Format
[015]mov qword ptr [rsp-0x8], rdi      ;Store rdi at rsp-8
[016]movzx eax, byte ptr [rdi]         ;Zero-extend byte at [rdi] into eax
[017]mov qword ptr [rsp-0x18], rax     ;Store extended value at rsp-24
[018]mov rax, qword ptr [rsp-0x18]     ;Load value from rsp-24 into rax
[019]lea rcx, ptr [rip 0x452af]        ;Load effective address (likely a jump table) into rcx
[020]movsxd rax, dword ptr [rcx rax*4] ;Load and sign-extend value from jump table into rax
[021]add rax, rcx                      ;Add base address to jump offset
[022]jmp rax                           ;Jump to the calculated address (switch case implementation)
[023]mov byte ptr [rsp-0x9], 0x0       ;Set byte at rsp-9 to 0 (default case in switch)
[024]jmp 0x558bdf671e65                ;Jump to function epilogue
[025]mov al, byte ptr [rsp-0x9]        ;Load byte from rsp-9 into al
[026]ret                               ;Return from function

;Set Rounding Mode
[027]movzx edi, al                ;Zero-extend al into edi (parameter for next function)
[028]call qword ptr [rip 0x5bf45] ;Call function pointer at rip+0x5bf45

;Rounding Mode Helper
[029]push rbp                     ;Save base pointer
[030]mov rbp, rsp                 ;Set new base pointer
[031]push rbx                     ;Save rbx register
[032]sub rsp, 0x8                 ;Allocate 8 bytes on stack
[033]mov ebx, edi                 ;Store first parameter in ebx
[034]mov rax, qword ptr fs:[0x0]  ;Load value from fs segment (thread-local storage)
[035]lea rax, ptr [rax-0x4f]      ;Calculate address at fs:0 - 79
[036]mov byte ptr [rax], bl       ;Store lowest byte of ebx at calculated address
[037]mov rbx, qword ptr [rbp-0x8] ;Restore rbx
[038]leave                        ;Restore stack and base pointer
[039]ret                          ;Return from function

;Set Rounding Mode (continued)
[040]pop rax ;Restore rax
[041]ret     ;Return from function

;Main Subtraction Function (continued)
[042]jmp 0x558bdf63473a            ;Jump to next part of subtraction function
[043]mov rax, qword ptr [rsp 0x8]  ;Load pointer to first Binary32 from rsp+8
[044]mov eax, dword ptr [rax]      ;Load value of first Binary32 into eax
[045]mov dword ptr [rsp 0x28], eax ;Store first Binary32 at rsp+40
[046]lea rdi, ptr [rsp 0x14]       ;Load address of second Binary32 (rsp+20) into rdi
[047]call 0x558bdf6346e0           ;Call function to borrow memory value

;Borrow Memory Value
[048]mov rax, rdi                 ;Copy input parameter to rax
[049]mov qword ptr [rsp-0x8], rax ;Store rax at rsp-8
[050]ret                          ;Return from function

;Main Subtraction Function (continued)
[051]mov qword ptr [rsp], rax      ;Store return value at top of stack
[052]jmp 0x558bdf634755            ;Jump to next part of subtraction
[053]mov rax, qword ptr [rsp]      ;Load pointer from stack into rax
[054]mov eax, dword ptr [rax]      ;Load value at pointer into eax
[055]mov dword ptr [rsp 0x2c], eax ;Store second Binary32 at rsp+44
[056]mov eax, dword ptr [rsp 0x28] ;Load first Binary32 into eax
[057]mov dword ptr [rsp 0x4c], eax ;Store first Binary32 at rsp+76
[058]mov edi, dword ptr [rsp 0x4c] ;Load first Binary32 into edi (first parameter)
[059]mov eax, dword ptr [rsp 0x2c] ;Load second Binary32 into eax
[060]mov dword ptr [rsp 0x50], eax ;Store second Binary32 at rsp+80
[061]mov esi, dword ptr [rsp 0x50] ;Load second Binary32 into esi (second parameter)
[062]call qword ptr [rip 0x9906b]  ;Call function for low-level float subtraction

;Low-level Float Subtraction
[063]push rbp            ;Save base pointer
[064]mov rbp, rsp        ;Set new base pointer
[065]mov eax, edi        ;Copy first Binary32 to eax
[066]mov edx, esi        ;Copy second Binary32 to edx
[067]xor esi, edi        ;XOR the two Binary32 values
[068]js 0x558bdf671e83   ;Jump if sign bit is set (signs are different)
[069]mov rsi, rdx        ;Copy second Binary32 to rsi
[070]mov rdi, rax        ;Copy first Binary32 to rdi
[071]call 0x558bdf6721a9 ;Call function to add float magnitudes

;Add Float Magnitudes
[072]push rbp            ;Save base pointer
[073]mov rbp, rsp        ;Set new base pointer
[074]mov rcx, rdi        ;Copy first Binary32 to rcx
[075]shr rcx, 0x17       ;Shift right by 23 bits (extract exponent)
[076]movzx ecx, cl       ;Zero-extend lowest byte to get 8-bit exponent
[077]mov r9, rcx         ;Copy exponent to r9
[078]mov r10, rdi        ;Copy first Binary32 to r10
[079]and r10d, 0x7fffff  ;Mask to get mantissa (23 bits)
[080]mov rax, rsi        ;Copy second Binary32 to rax
[081]shr rax, 0x17       ;Shift right by 23 bits (extract exponent)
[082]movzx eax, al       ;Zero-extend lowest byte to get 8-bit exponent
[083]mov rdx, rsi        ;Copy second Binary32 to rdx
[084]and edx, 0x7fffff   ;Mask to get mantissa (23 bits)
[085]mov r11, rcx        ;Copy first exponent to r11
[086]sub r11, rax        ;Subtract exponents (for comparing magnitude)
[087]jnz 0x558bdf67224f  ;Jump if exponents are different
[088]mov r8d, edi        ;Copy first Binary32 to r8d
[089]shr r8d, 0x1f       ;Shift right by 31 bits (extract sign bit)
[090]shl r10, 0x6        ;Shift first mantissa left by 6 bits (normalize)
[091]shl rdx, 0x6        ;Shift second mantissa left by 6 bits (normalize)
[092]test r11, r11       ;Test if exponent difference is zero
[093]js 0x558bdf6722c4   ;Jump if exponent difference is negative
[094]cmp rax, 0xff       ;Compare second exponent with 0xFF (check for NaN/Infinity)
[095]jz 0x558bdf67230b   ;Jump if second number is NaN or Infinity
[096]test rcx, rcx       ;Test if first exponent is zero
[097]mov esi, 0x20000000 ;Set esi to 0x20000000 (implied 1 for normalized numbers)
[098]cmovz rsi, r10      ;If first exponent is zero, use mantissa directly (denormal)
[099]mov rdi, rax        ;Copy second exponent to rdi
[100]sub rdi, rcx        ;Calculate exponent difference
[101]add rsi, r10        ;Add implied 1 to first mantissa
[102]cmp rdi, 0x1e       ;Compare exponent difference with 30
[103]jnbe 0x558bdf672327 ;Jump if difference is not within range
[104]mov ecx, edi        ;Copy exponent difference to ecx
[105]neg ecx             ;Negate exponent difference
[106]mov r9d, esi        ;Copy first mantissa with implied 1 to r9d
[107]shl r9d, cl         ;Shift left by exponent difference
[108]test r9d, r9d       ;Test if shifted value is zero
[109]setnz r10b          ;Set r10b to 1 if not zero, 0 if zero
[110]movzx r10d, r10b    ;Zero-extend r10b to r10d
[111]mov ecx, edi        ;Copy exponent difference to ecx
[112]shr esi, cl         ;Shift first mantissa right by exponent difference
[113]or r10d, esi        ;Combine shifted bits with sticky bit
[114]mov r10d, r10d      ;Clear upper 32 bits of r10
[115]mov r9, rax         ;Copy second exponent to r9
[116]jmp 0x558bdf6722a3  ;Jump to next part of float addition
[117]lea rdx, ptr [r10 rdx*1 0x20000000] ;Calculate sum of mantissas with implied 1
[118]cmp rdx, 0x3fffffff ;Check if result needs normalization
[119]jnbe 0x558bdf67220e ;Jump if normalization needed
[120]movzx edi, r8b      ;Zero-extend sign bit to edi
[121]mov rsi, r9         ;Copy exponent to rsi
[122]call 0x558bdf671f65 ;Call function to round and pack to Binary32 format

;Round and Pack to Binary32 Format
[123]push rbp                      ;Save base pointer
[124]mov rbp, rsp                  ;Set new base pointer
[125]push r15                      ;Save r15 register
[126]push r14                      ;Save r14 register
[127]push r13                      ;Save r13 register
[128]push r12                      ;Save r12 register
[129]push rbx                      ;Save rbx register
[130]sub rsp, 0x18                 ;Allocate 24 bytes on stack
[131]mov r15d, edi                 ;Store sign bit in r15d
[132]mov dword ptr [rbp-0x34], edi ;Store sign bit at rbp-52
[133]mov rbx, rsi                  ;Store exponent in rbx
[134]mov r13, rdx                  ;Store mantissa in r13
[135]mov rax, qword ptr fs:[0x0]   ;Load value from fs segment (thread-local storage)
[136]lea rax, ptr [rax-0x4f]       ;Calculate address at fs:0 - 79
[137]movzx r14d, byte ptr [rax]    ;Load rounding mode into r14d
[138]test r14b, r14b               ;Test if rounding mode is 0 (to nearest)
[139]setz byte ptr [rbp-0x40]      ;Set flag at rbp-64 if rounding mode is 0
[140]mov r12d, 0x40                ;Set r12d to 64 (used for rounding)
[141]test r14b, 0xfb               ;Test rounding mode against 0xfb
[142]jz 0x558bdf671fc9             ;Jump if specific rounding mode
[143]mov r15d, r13d                ;Copy mantissa to r15d
[144]and r15d, 0x7f                ;Mask with 0x7f (get 7 least significant bits)
[145]mov dword ptr [rbp-0x38], ebx ;Store exponent at rbp-56
[146]cmp ebx, 0xfc                 ;Compare exponent with 0xfc
[147]jbe 0x558bdf672000            ;Jump if exponent <= 0xfc
[148]movzx r12d, r12b              ;Zero-extend r12b to r12d
[149]add r12, r13                  ;Add mantissa to r12
[150]shr r12, 0x7                  ;Shift right by 7 bits (rounding)
[151]test r15b, r15b               ;Test if lower 7 bits are zero
[152]jnz 0x558bdf6720ec            ;Jump if lower 7 bits are not zero
[153]mov rax, qword ptr fs:[0x0]   ;Load value from fs segment
[154]lea rax, ptr [rax-0x50]       ;Calculate address at fs:0 - 80
[155]or byte ptr [rax], 0x1        ;Set inexact flag
[156]cmp r14b, 0x6                 ;Compare rounding mode with 6
[157]jnz 0x558bdf672014            ;Jump if rounding mode is not 6
[158]cmp r15b, 0x40                ;Compare lower 7 bits with 0x40
[159]setz al                       ;Set al to 1 if equal, 0 if not
[160]movzx eax, al                 ;Zero-extend al to eax
[161]and rax, qword ptr [rbp-0x40] ;AND with rounding flag
[162]not rax                       ;Complement result
[163]and r12, rax                  ;Mask r12 with result
[164]mov eax, 0x0                  ;Set eax to 0
[165]cmovz rbx, rax                ;If zero, set exponent to 0
[166]mov eax, dword ptr [rbp-0x34] ;Load sign bit into eax
[167]shl eax, 0x1f                 ;Shift sign bit to position 31
[168]shl ebx, 0x17                 ;Shift exponent to position 23-30
[169]add eax, ebx                  ;Combine sign and exponent
[170]mov eax, eax                  ;Clear upper 32 bits
[171]add rax, r12                  ;Add mantissa to result
[172]add rsp, 0x18                 ;Deallocate stack space
[173]pop rbx                       ;Restore rbx
[174]pop r12                       ;Restore r12
[175]pop r13                       ;Restore r13
[176]pop r14                       ;Restore r14
[177]pop r15                       ;Restore r15
[178]pop rbp                       ;Restore base pointer
[179]ret                           ;Return from function

;Add Float Magnitudes (epilogue)
[180]pop rbp ;Restore base pointer
[181]ret     ;Return from function

;Low-level Float Subtraction (epilogue)
[182]jmp 0x558bdf671e81 ;Jump to another part of function
[183]pop rbp            ;Restore base pointer
[184]ret                ;Return from function

;Main Subtraction Function (epilogue)
[185]mov dword ptr [rsp 0x54], eax ;Store result at rsp+84
[186]mov eax, dword ptr [rsp 0x54] ;Load result back into eax
[187]mov dword ptr [rsp 0x24], eax ;Store result at rsp+36
[188]mov eax, dword ptr [rsp 0x24] ;Load result back into eax
[189]mov dword ptr [rsp 0x20], eax ;Store result at rsp+32
[190]mov eax, dword ptr [rsp 0x20] ;Load result back into eax
[191]add rsp, 0x58                 ;Deallocate 88 bytes from stack
[192]ret                           ;Return from function