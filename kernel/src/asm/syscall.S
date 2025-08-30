# Syscall Entry Trampoline for Polymera OS
# 
# This file contains the assembly code for handling system calls via int 0x80.
# The trampoline saves registers, calls the syscall dispatcher, restores
# registers, and returns to user space.

.section .text
.global syscall_entry
.global syscall_entry_end

# Syscall entry point for int 0x80
# This is the main entry point for all system calls
syscall_entry:
    # Save all general purpose registers
    # Note: We don't need to save rsp as it's already saved by the interrupt
    pushq %rax      # Save syscall number
    pushq %rcx      # Save syscall arguments
    pushq %rdx
    pushq %rsi
    pushq %rdi
    pushq %r8
    pushq %r9
    pushq %r10
    pushq %r11
    pushq %rbx
    pushq %rbp
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    
    # Save segment registers and flags
    pushq %ds
    pushq %es
    pushq %fs
    pushq %gs
    
    # Set up kernel data segments
    movq $0x10, %rax    # Kernel data segment selector
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    
    # Prepare arguments for syscall dispatcher
    # syscall number is in the saved rax on stack
    # arguments are in rdi, rsi, rdx, r10, r8, r9 (syscall calling convention)
    movq %rdi, %rdi     # First argument (a0)
    movq %rsi, %rsi     # Second argument (a1)
    movq %rdx, %rdx     # Third argument (a2)
    movq %r10, %rcx     # Fourth argument (a3)
    
    # Get syscall number from saved rax on stack
    movq 120(%rsp), %rax  # rax is saved at offset 120 (15 * 8)
    
    # Call the syscall dispatcher
    # The dispatcher expects: dispatch(syscall_num, a0, a1, a2, a3)
    callq handlers_dispatch
    
    # Store return value in saved rax location
    movq %rax, 120(%rsp)
    
    # Restore segment registers
    popq %gs
    popq %fs
    popq %es
    popq %ds
    
    # Restore general purpose registers
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbp
    popq %rbx
    popq %r11
    popq %r10
    popq %r9
    popq %r8
    popq %rdi
    popq %rsi
    popq %rdx
    popq %rcx
    popq %rax
    
    # Return to user space
    iretq

# Alternative syscall entry using syscall instruction (AMD64)
# This provides a faster path for 64-bit system calls
.global syscall_entry_syscall
syscall_entry_syscall:
    # Save registers (syscall instruction doesn't save rcx and r11)
    pushq %rcx      # Save return address
    pushq %r11      # Save flags
    pushq %rax      # Save syscall number
    pushq %rdi      # Save arguments
    pushq %rsi
    pushq %rdx
    pushq %r10
    pushq %r8
    pushq %r9
    pushq %rbx
    pushq %rbp
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    
    # Save segment registers
    pushq %ds
    pushq %es
    pushq %fs
    pushq %gs
    
    # Set up kernel data segments
    movq $0x10, %rax    # Kernel data segment selector
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    
    # Prepare arguments for dispatcher
    movq %rdi, %rdi     # a0
    movq %rsi, %rsi     # a1
    movq %rdx, %rdx     # a2
    movq %r10, %rcx     # a3
    
    # Get syscall number from saved rax
    movq 120(%rsp), %rax
    
    # Call dispatcher
    callq handlers_dispatch
    
    # Store return value
    movq %rax, 120(%rsp)
    
    # Restore segment registers
    popq %gs
    popq %fs
    popq %es
    popq %ds
    
    # Restore general purpose registers
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbp
    popq %rbx
    popq %r9
    popq %r8
    popq %r10
    popq %rdx
    popq %rsi
    popq %rdi
    popq %rax
    popq %r11
    popq %rcx
    
    # Return to user space
    sysretq

# Syscall entry end marker for size calculation
syscall_entry_end:

# Data section for syscall information
.section .data
.global syscall_entry_size
syscall_entry_size:
    .quad syscall_entry_end - syscall_entry

# Syscall number definitions
# These match the constants defined in the Rust code
.equ SYS_YIELD, 1
.equ SYS_EXIT, 2
.equ SYS_SEND, 3
.equ SYS_RECV, 4
.equ SYS_CHAN_CREATE, 5
.equ SYS_STATS, 6
.equ SYS_DEBUG, 7

# Syscall entry point table (for potential future use)
.global syscall_table
syscall_table:
    .quad syscall_entry      # Default entry point
    .quad syscall_entry_syscall  # Fast syscall entry point

# Export syscall entry points for Rust code
.global syscall_entry_ptr
.global syscall_entry_syscall_ptr
syscall_entry_ptr:
    .quad syscall_entry
syscall_entry_syscall_ptr:
    .quad syscall_entry_syscall
