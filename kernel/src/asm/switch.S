# Context Switch Assembly Implementation for x86_64
# 
# This file implements the low-level context switching mechanism for the
# Polymera OS scheduler. It saves and restores CPU registers to switch
# between task execution contexts.
#
# Function: context_switch(old: *mut CpuContext, new: *const CpuContext)
# 
# Arguments:
#   RDI - Pointer to current task's context (will be updated)
#   RSI - Pointer to new task's context (will be loaded)
#
# The CpuContext structure layout (matching context.rs):
#   Offset 0x00: RIP (instruction pointer)
#   Offset 0x08: RSP (stack pointer)  
#   Offset 0x10: RBX
#   Offset 0x18: RBP
#   Offset 0x20: R12
#   Offset 0x28: R13
#   Offset 0x30: R14
#   Offset 0x38: R15

.section .text
.global context_switch
.type context_switch, @function

context_switch:
    # Save current context to *old (RDI points to CpuContext)
    
    # Save current instruction pointer (return address from call)
    # The return address is on the stack, so we save it as RIP
    mov (%rsp), %rax
    mov %rax, 0x00(%rdi)
    
    # Save current stack pointer (after adjusting for return address)
    lea 8(%rsp), %rax
    mov %rax, 0x08(%rdi)
    
    # Save callee-saved registers
    mov %rbx, 0x10(%rdi)    # Save RBX
    mov %rbp, 0x18(%rdi)    # Save RBP
    mov %r12, 0x20(%rdi)    # Save R12
    mov %r13, 0x28(%rdi)    # Save R13
    mov %r14, 0x30(%rdi)    # Save R14
    mov %r15, 0x38(%rdi)    # Save R15
    
    # Load new context from *new (RSI points to CpuContext)
    
    # Restore callee-saved registers
    mov 0x10(%rsi), %rbx    # Restore RBX
    mov 0x18(%rsi), %rbp    # Restore RBP
    mov 0x20(%rsi), %r12    # Restore R12
    mov 0x28(%rsi), %r13    # Restore R13
    mov 0x30(%rsi), %r14    # Restore R14
    mov 0x38(%rsi), %r15    # Restore R15
    
    # Switch to new stack
    mov 0x08(%rsi), %rsp    # Load new stack pointer
    
    # Jump to new instruction pointer (resume execution)
    jmp *0x00(%rsi)         # Jump to new RIP
    
.size context_switch, .-context_switch

# Alternative implementation with stack frame setup
# This version is more explicit about the calling convention
.global context_switch_with_frame
.type context_switch_with_frame, @function

context_switch_with_frame:
    # Standard function prologue
    push %rbp
    mov %rsp, %rbp
    
    # Arguments are in RDI (old) and RSI (new)
    # Save all callee-saved registers to old context
    
    # Calculate the return address (after we restore the stack)
    lea return_point(%rip), %rax
    mov %rax, 0x00(%rdi)        # Store return point as RIP
    
    # Save stack pointer (will be restored in new context)
    mov %rbp, 0x08(%rdi)        # Save current RBP as RSP
    
    # Save callee-saved registers
    mov %rbx, 0x10(%rdi)
    mov %rbp, 0x18(%rdi)
    mov %r12, 0x20(%rdi)
    mov %r13, 0x28(%rdi)
    mov %r14, 0x30(%rdi)
    mov %r15, 0x38(%rdi)
    
    # Load new context
    mov 0x10(%rsi), %rbx        # Restore RBX
    mov 0x18(%rsi), %rbp        # Restore RBP
    mov 0x20(%rsi), %r12        # Restore R12
    mov 0x28(%rsi), %r13        # Restore R13
    mov 0x30(%rsi), %r14        # Restore R14
    mov 0x38(%rsi), %r15        # Restore R15
    
    # Switch stack and jump to new context
    mov 0x08(%rsi), %rsp        # Load new stack
    jmp *0x00(%rsi)             # Jump to new RIP

return_point:
    # Execution resumes here when this context is switched back in
    pop %rbp                    # Restore frame pointer
    ret                         # Return to caller

.size context_switch_with_frame, .-context_switch_with_frame

# Simplified context switch for kernel threads
# This version assumes we're switching between kernel contexts only
.global kernel_context_switch
.type kernel_context_switch, @function

kernel_context_switch:
    # Save minimal state for kernel threads
    pushfq                      # Save flags
    
    # Save return address as RIP
    mov 8(%rsp), %rax          # Get return address (after pushfq)
    mov %rax, 0x00(%rdi)
    
    # Save stack pointer
    mov %rsp, 0x08(%rdi)
    
    # Save callee-saved registers
    mov %rbx, 0x10(%rdi)
    mov %rbp, 0x18(%rdi)
    mov %r12, 0x20(%rdi)
    mov %r13, 0x28(%rdi)
    mov %r14, 0x30(%rdi)
    mov %r15, 0x38(%rdi)
    
    # Load new context
    mov 0x10(%rsi), %rbx
    mov 0x18(%rsi), %rbp
    mov 0x20(%rsi), %r12
    mov 0x28(%rsi), %r13
    mov 0x30(%rsi), %r14
    mov 0x38(%rsi), %r15
    
    # Switch to new stack
    mov 0x08(%rsi), %rsp
    
    # Restore flags and jump
    popfq                       # Restore flags from new stack
    jmp *0x00(%rsi)
    
.size kernel_context_switch, .-kernel_context_switch

# Task startup trampoline
# This function is used as the entry point for new tasks
.global task_startup_trampoline
.type task_startup_trampoline, @function

task_startup_trampoline:
    # New tasks start here with their entry point in R12
    # and argument in R13 (if any)
    
    # Set up initial stack frame
    xor %rbp, %rbp              # Clear frame pointer (start of call chain)
    
    # Call the task's main function
    mov %r13, %rdi              # Move argument to first parameter register
    call *%r12                  # Call the task function
    
    # If the task function returns, we need to terminate the task
    # In a full implementation, this would call exit_task()
    mov $60, %rax               # sys_exit system call number
    mov $0, %rdi                # Exit status
    syscall                     # Make system call (placeholder)
    
    # Should never reach here, but just in case
    hlt
    
.size task_startup_trampoline, .-task_startup_trampoline

# Context switch statistics helper
# Increments a counter for performance monitoring
.global context_switch_counter
.section .data
context_switch_count: .quad 0

.section .text
.global increment_context_switch_counter
.type increment_context_switch_counter, @function

increment_context_switch_counter:
    # Atomically increment the context switch counter
    lock incq context_switch_count(%rip)
    ret

.size increment_context_switch_counter, .-increment_context_switch_counter

.global get_context_switch_count
.type get_context_switch_count, @function

get_context_switch_count:
    # Return the current context switch count
    mov context_switch_count(%rip), %rax
    ret

.size get_context_switch_count, .-get_context_switch_count

# Utility function to get current stack pointer
.global get_current_stack_pointer
.type get_current_stack_pointer, @function

get_current_stack_pointer:
    mov %rsp, %rax
    ret

.size get_current_stack_pointer, .-get_current_stack_pointer

# Utility function to get current instruction pointer
.global get_current_instruction_pointer
.type get_current_instruction_pointer, @function

get_current_instruction_pointer:
    mov (%rsp), %rax            # Return address is current IP
    ret

.size get_current_instruction_pointer, .-get_current_instruction_pointer

# Debug helper to save all registers to a buffer
.global save_all_registers
.type save_all_registers, @function

save_all_registers:
    # RDI points to a buffer to store all registers
    # Buffer layout: RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP, R8-R15
    
    mov %rax, 0x00(%rdi)
    mov %rbx, 0x08(%rdi)
    mov %rcx, 0x10(%rdi)
    mov %rdx, 0x18(%rdi)
    mov %rsi, 0x20(%rdi)
    # Skip RDI since it's our buffer pointer
    mov %rbp, 0x30(%rdi)
    mov %rsp, 0x38(%rdi)
    mov %r8,  0x40(%rdi)
    mov %r9,  0x48(%rdi)
    mov %r10, 0x50(%rdi)
    mov %r11, 0x58(%rdi)
    mov %r12, 0x60(%rdi)
    mov %r13, 0x68(%rdi)
    mov %r14, 0x70(%rdi)
    mov %r15, 0x78(%rdi)
    
    ret

.size save_all_registers, .-save_all_registers

# Assembly implementation notes:
# 
# 1. Register Preservation:
#    - Caller-saved: RAX, RCX, RDX, RSI, RDI, R8-R11
#    - Callee-saved: RBX, RBP, R12-R15 (these are what we preserve)
#    - Special: RSP, RIP (stack and instruction pointers)
#
# 2. Function Calling Convention (System V ABI):
#    - Arguments: RDI, RSI, RDX, RCX, R8, R9
#    - Return value: RAX
#    - Caller must save: RAX, RCX, RDX, RSI, RDI, R8-R11
#    - Callee must save: RBX, RBP, R12-R15
#
# 3. Stack Layout:
#    - Grows downward (toward lower addresses)
#    - Must be 16-byte aligned before function calls
#    - Return address is pushed by CALL instruction
#
# 4. Context Switch Process:
#    - Save current CPU state to old context
#    - Load new CPU state from new context
#    - Jump to new instruction pointer
#    - Execution continues in new task
#
# 5. Task Startup:
#    - New tasks need special setup since they don't have saved state
#    - Use task_startup_trampoline for initial execution
#    - Set up proper stack frame and calling convention
