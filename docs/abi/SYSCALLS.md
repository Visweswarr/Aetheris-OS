# Polymera OS System Call Interface

**Version**: 1.0  
**Date**: December 2024  
**Status**: 🔒 **STABLE**  
**ABI**: x86_64 Linux-style calling convention  

## 📋 **Overview**

This document defines the stable system call interface for Polymera OS. All syscall numbers, calling conventions, and error codes are defined here and must not change without proper versioning and migration procedures.

**⚠️ IMPORTANT**: Changes to this document require:
1. Version bump and migration documentation
2. CI validation that all generated artifacts are updated
3. User space compatibility verification
4. Kernel implementation updates

## 🏗️ **Calling Convention**

### **x86_64 Linux-style ABI**
- **System Call Number**: `rax` register
- **Arguments**: `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9` (in order)
- **Return Value**: `rax` register
- **Error Handling**: Negative return values indicate errors
- **Clobbered Registers**: `rcx`, `r11` (syscall instruction)

### **Assembly Example**
```nasm
; Example: syscall(1, arg1, arg2, arg3)
mov rax, 1          ; syscall number
mov rdi, arg1       ; first argument
mov rsi, arg2       ; second argument
mov rdx, arg3       ; third argument
syscall              ; make system call
; rax contains return value
```

### **C Function Prototypes**
```c
// Generated automatically from this document
long syscall(long number, ...);
```

## 🔢 **System Call Numbers**

### **Core System Calls (0-99)**

| Number | Name | Description | Arguments | Return Value |
|--------|------|-------------|-----------|--------------|
| 0 | `SYS_EXIT` | Terminate process | `int status` | Never returns |
| 1 | `SYS_READ` | Read from file descriptor | `int fd, void *buf, size_t count` | Bytes read or error |
| 2 | `SYS_WRITE` | Write to file descriptor | `int fd, const void *buf, size_t count` | Bytes written or error |
| 3 | `SYS_OPEN` | Open file | `const char *pathname, int flags, mode_t mode` | File descriptor or error |
| 4 | `SYS_CLOSE` | Close file descriptor | `int fd` | 0 on success, error on failure |
| 5 | `SYS_STAT` | Get file status | `const char *pathname, struct stat *statbuf` | 0 on success, error on failure |
| 6 | `SYS_FSTAT` | Get file status by fd | `int fd, struct stat *statbuf` | 0 on success, error on failure |
| 7 | `SYS_DEBUG` | Debug operations | `int op, long arg1, long arg2, long arg3` | Operation result |
| 8 | `SYS_MMAP` | Map memory | `void *addr, size_t length, int prot, int flags, int fd, off_t offset` | Mapped address or error |
| 9 | `SYS_MUNMAP` | Unmap memory | `void *addr, size_t length` | 0 on success, error on failure |
| 10 | `SYS_BRK` | Change data segment size | `void *addr` | New break address or error |

### **Process Management (100-199)**

| Number | Name | Description | Arguments | Return Value |
|--------|------|-------------|-----------|--------------|
| 100 | `SYS_FORK` | Create child process | None | Process ID or error |
| 101 | `SYS_EXECVE` | Execute program | `const char *pathname, char *const argv[], char *const envp[]` | Never returns on success |
| 102 | `SYS_WAITPID` | Wait for process | `pid_t pid, int *wstatus, int options` | Process ID or error |
| 103 | `SYS_GETPID` | Get process ID | None | Process ID |
| 104 | `SYS_GETPPID` | Get parent process ID | None | Parent process ID |
| 105 | `SYS_SETUID` | Set user ID | `uid_t uid` | 0 on success, error on failure |
| 106 | `SYS_GETUID` | Get user ID | None | User ID |
| 107 | `SYS_SETGID` | Set group ID | `gid_t gid` | 0 on success, error on failure |
| 108 | `SYS_GETGID` | Get group ID | None | Group ID |

### **IPC System Calls (200-299)**

| Number | Name | Description | Arguments | Return Value |
|--------|------|-------------|-----------|--------------|
| 200 | `SYS_IPC_SEND` | Send IPC message | `ipc_token_t token, const void *data, size_t size, uint64_t flags` | Message ID or error |
| 201 | `SYS_IPC_RECV` | Receive IPC message | `ipc_token_t token, void *data, size_t size, uint64_t flags` | Message ID or error |
| 202 | `SYS_IPC_CREATE_CHANNEL` | Create IPC channel | `const char *name, uint64_t flags` | Channel ID or error |
| 203 | `SYS_IPC_DESTROY_CHANNEL` | Destroy IPC channel | `ipc_channel_t channel` | 0 on success, error on failure |
| 204 | `SYS_IPC_GRANT_TOKEN` | Grant IPC token | `ipc_channel_t channel, pid_t target_pid, uint64_t permissions` | Token ID or error |
| 205 | `SYS_IPC_REVOKE_TOKEN` | Revoke IPC token | `ipc_token_t token` | 0 on success, error on failure |
| 206 | `SYS_IPC_POLL` | Poll IPC channels | `ipc_poll_t *polls, size_t count, int timeout` | Number of ready channels or error |

### **Memory Management (300-399)**

| Number | Name | Description | Arguments | Return Value |
|--------|------|-------------|-----------|--------------|
| 300 | `SYS_MEM_ALLOC` | Allocate memory | `size_t size, uint64_t flags` | Memory address or error |
| 301 | `SYS_MEM_FREE` | Free memory | `void *ptr` | 0 on success, error on failure |
| 302 | `SYS_MEM_PROTECT` | Change memory protection | `void *addr, size_t length, int prot` | 0 on success, error on failure |
| 303 | `SYS_MEM_SYNC` | Synchronize memory | `void *addr, size_t length, int flags` | 0 on success, error on failure |
| 304 | `SYS_MEM_LOCK` | Lock memory in RAM | `void *addr, size_t length` | 0 on success, error on failure |
| 305 | `SYS_MEM_UNLOCK` | Unlock memory | `void *addr, size_t length` | 0 on success, error on failure |

### **Security & Capabilities (400-499)**

| Number | Name | Description | Arguments | Return Value |
|--------|------|-------------|-----------|--------------|
| 400 | `SYS_CAP_GRANT` | Grant capability | `pid_t target_pid, cap_id_t cap_id, uint64_t permissions` | 0 on success, error on failure |
| 401 | `SYS_CAP_REVOKE` | Revoke capability | `cap_id_t cap_id` | 0 on success, error on failure |
| 402 | `SYS_CAP_CHECK` | Check capability | `cap_id_t cap_id, uint64_t operation` | 0 if allowed, error if denied |
| 403 | `SYS_CAP_QUERY` | Query capability info | `cap_id_t cap_id, cap_info_t *info` | 0 on success, error on failure |
| 404 | `SYS_AUDIT_LOG` | Log audit event | `const char *event, const void *data, size_t size` | 0 on success, error on failure |

### **Scheduling & Timing (500-599)**

| Number | Name | Description | Arguments | Return Value |
|--------|------|-------------|-----------|--------------|
| 500 | `SYS_SCHED_YIELD` | Yield CPU | None | 0 |
| 501 | `SYS_SCHED_SET_PRIORITY` | Set process priority | `int priority` | 0 on success, error on failure |
| 502 | `SYS_SCHED_GET_PRIORITY` | Get process priority | None | Priority value or error |
| 503 | `SYS_SCHED_SET_POLICY` | Set scheduling policy | `int policy` | 0 on success, error on failure |
| 504 | `SYS_SCHED_GET_POLICY` | Get scheduling policy | None | Policy value or error |
| 505 | `SYS_SLEEP` | Sleep for specified time | `uint64_t milliseconds` | 0 on success, error on failure |
| 506 | `SYS_GET_TIME` | Get current time | `time_t *seconds, long *nanoseconds` | 0 on success, error on failure |

### **File System (600-699)**

| Number | Name | Description | Arguments | Return Value |
|--------|------|-------------|-----------|--------------|
| 600 | `SYS_CHDIR` | Change directory | `const char *path` | 0 on success, error on failure |
| 601 | `SYS_GETCWD` | Get current directory | `char *buf, size_t size` | 0 on success, error on failure |
| 602 | `SYS_MKDIR` | Create directory | `const char *pathname, mode_t mode` | 0 on success, error on failure |
| 603 | `SYS_RMDIR` | Remove directory | `const char *pathname` | 0 on success, error on failure |
| 604 | `SYS_UNLINK` | Remove file | `const char *pathname` | 0 on success, error on failure |
| 605 | `SYS_LINK` | Create hard link | `const char *oldpath, const char *newpath` | 0 on success, error on failure |
| 606 | `SYS_SYMLINK` | Create symbolic link | `const char *target, const char *linkpath` | 0 on success, error on failure |
| 607 | `SYS_READLINK` | Read symbolic link | `const char *pathname, char *buf, size_t bufsiz` | Bytes read or error |

### **Network (700-799)**

| Number | Name | Description | Arguments | Return Value |
|--------|------|-------------|-----------|--------------|
| 700 | `SYS_SOCKET` | Create socket | `int domain, int type, int protocol` | File descriptor or error |
| 701 | `SYS_BIND` | Bind socket to address | `int sockfd, const struct sockaddr *addr, socklen_t addrlen` | 0 on success, error on failure |
| 702 | `SYS_CONNECT` | Connect socket | `int sockfd, const struct sockaddr *addr, socklen_t addrlen` | 0 on success, error on failure |
| 703 | `SYS_LISTEN` | Listen for connections | `int sockfd, int backlog` | 0 on success, error on failure |
| 704 | `SYS_ACCEPT` | Accept connection | `int sockfd, struct sockaddr *addr, socklen_t *addrlen` | File descriptor or error |
| 705 | `SYS_SEND` | Send data on socket | `int sockfd, const void *buf, size_t len, int flags` | Bytes sent or error |
| 706 | `SYS_RECV` | Receive data from socket | `int sockfd, void *buf, size_t len, int flags` | Bytes received or error |

### **System Information (800-899)**

| Number | Name | Description | Arguments | Return Value |
|--------|------|-------------|-----------|--------------|
| 800 | `SYS_UNAME` | Get system information | `struct utsname *buf` | 0 on success, error on failure |
| 801 | `SYS_SYSINFO` | Get system statistics | `struct sysinfo *info` | 0 on success, error on failure |
| 802 | `SYS_GETRUSAGE` | Get resource usage | `int who, struct rusage *usage` | 0 on success, error on failure |
| 803 | `SYS_TIMES` | Get process times | `struct tms *buf` | Clock ticks or error |
| 804 | `SYS_GETUID` | Get real user ID | None | User ID |
| 805 | `SYS_GETEUID` | Get effective user ID | None | Effective user ID |
| 806 | `SYS_GETGID` | Get real group ID | None | Group ID |
| 807 | `SYS_GETEGID` | Get effective group ID | None | Effective group ID |

### **Experimental & Reserved (900-999)**

| Number | Name | Description | Arguments | Return Value |
|--------|------|-------------|-----------|--------------|
| 900 | `SYS_EXPERIMENTAL_1` | Reserved for experimental use | Varies | Varies |
| 901 | `SYS_EXPERIMENTAL_2` | Reserved for experimental use | Varies | Varies |
| 999 | `SYS_RESERVED` | Reserved for future use | None | -ENOSYS |

## 🚨 **Error Codes**

### **Standard Error Numbers**

| Error Code | Name | Description |
|------------|------|-------------|
| -1 | `-EPERM` | Operation not permitted |
| -2 | `-ENOENT` | No such file or directory |
| -3 | `-ESRCH` | No such process |
| -4 | `-EINTR` | Interrupted system call |
| -5 | `-EIO` | Input/output error |
| -6 | `-ENXIO` | No such device or address |
| -7 | `-E2BIG` | Argument list too long |
| -8 | `-ENOEXEC` | Exec format error |
| -9 | `-EBADF` | Bad file descriptor |
| -10 | `-ECHILD` | No child processes |
| -11 | `-EAGAIN` | Resource temporarily unavailable |
| -12 | `-ENOMEM` | Cannot allocate memory |
| -13 | `-EACCES` | Permission denied |
| -14 | `-EFAULT` | Bad address |
| -15 | `-ENOTBLK` | Block device required |
| -16 | `-EBUSY` | Device or resource busy |
| -17 | `-EEXIST` | File exists |
| -18 | `-EXDEV` | Invalid cross-device link |
| -19 | `-ENODEV` | No such device |
| -20 | `-ENOTDIR` | Not a directory |
| -21 | `-EISDIR` | Is a directory |
| -22 | `-EINVAL` | Invalid argument |
| -23 | `-ENFILE` | Too many open files in system |
| -24 | `-EMFILE` | Too many open files |
| -25 | `-ENOTTY` | Inappropriate ioctl for device |
| -26 | `-ETXTBSY` | Text file busy |
| -27 | `-EFBIG` | File too large |
| -28 | `-ENOSPC` | No space left on device |
| -29 | `-ESPIPE` | Illegal seek |
| -30 | `-EROFS` | Read-only file system |
| -31 | `-EMLINK` | Too many links |
| -32 | `-EPIPE` | Broken pipe |
| -33 | `-EDOM` | Numerical argument out of domain |
| -34 | `-ERANGE` | Numerical result out of range |
| -35 | `-EDEADLK` | Resource deadlock avoided |
| -36 | `-ENAMETOOLONG` | File name too long |
| -37 | `-ENOLCK` | No locks available |
| -38 | `-ENOSYS` | Function not implemented |
| -39 | `-ENOTEMPTY` | Directory not empty |
| -40 | `-ELOOP` | Too many levels of symbolic links |

### **Polymera OS Specific Errors**

| Error Code | Name | Description |
|------------|------|-------------|
| -1000 | `-EPOLYMERA_BASE` | Base for Polymera OS specific errors |
| -1001 | `-EIPC_INVALID_TOKEN` | Invalid IPC token |
| -1002 | `-EIPC_CHANNEL_FULL` | IPC channel is full |
| -1003 | `-EIPC_CHANNEL_EMPTY` | IPC channel is empty |
| -1004 | `-EIPC_PERMISSION_DENIED` | IPC permission denied |
| -1005 | `-ECAP_INVALID` | Invalid capability |
| -1006 | `-ECAP_EXPIRED` | Capability has expired |
| -1007 | `-ECAP_INSUFFICIENT` | Insufficient capability permissions |
| -1008 | `-EAUDIT_FAILED` | Audit logging failed |
| -1009 | `-ESCHED_INVALID_PRIORITY` | Invalid scheduling priority |
| -1010 | `-ESCHED_INVALID_POLICY` | Invalid scheduling policy |

## 📊 **Data Types**

### **Basic Types**
```c
typedef int64_t syscall_t;           // System call return type
typedef uint64_t ipc_token_t;        // IPC token identifier
typedef uint64_t ipc_channel_t;      // IPC channel identifier
typedef uint64_t cap_id_t;           // Capability identifier
typedef uint64_t pid_t;              // Process identifier
typedef uint64_t uid_t;              // User identifier
typedef uint64_t gid_t;              // Group identifier
typedef uint64_t mode_t;             // File mode/permissions
typedef uint64_t size_t;             // Size type
typedef int64_t off_t;               // Offset type
typedef uint64_t time_t;             // Time type
```

### **IPC Structures**
```c
struct ipc_message {
    uint64_t id;                     // Message identifier
    uint64_t sender_pid;             // Sender process ID
    uint64_t timestamp;              // Message timestamp
    uint64_t flags;                  // Message flags
    uint8_t data[];                  // Variable length data
};

struct ipc_poll {
    ipc_channel_t channel;           // Channel to poll
    uint64_t events;                 // Events to watch for
    uint64_t revents;                // Events that occurred
};
```

### **Capability Structures**
```c
struct cap_info {
    cap_id_t id;                     // Capability identifier
    uint64_t target;                 // Target resource
    uint64_t permissions;            // Granted permissions
    uint64_t expires_at;             // Expiration timestamp
    uint64_t granted_by;             // Granting process ID
};
```

## 🔒 **Stability Guarantees**

### **ABI Stability**
- **System Call Numbers**: Never change once assigned
- **Calling Convention**: Stable across kernel versions
- **Error Codes**: Standard codes remain stable
- **Data Structures**: Layout changes require versioning

### **Versioning Policy**
- **Major Version**: Breaking changes require new syscall numbers
- **Minor Version**: New syscalls can be added
- **Patch Version**: Bug fixes only, no interface changes

### **Migration Strategy**
- **Deprecation**: Old syscalls marked deprecated for 2 major versions
- **Compatibility**: Kernel maintains backward compatibility
- **Documentation**: Migration guides provided for breaking changes

## 🧪 **Testing & Validation**

### **ABI Compliance Tests**
- **Calling Convention**: Verify argument passing and return values
- **Error Handling**: Test all error conditions and codes
- **Data Structure Layout**: Validate struct sizes and alignment
- **Edge Cases**: Test boundary conditions and invalid inputs

### **Performance Benchmarks**
- **Syscall Overhead**: Measure baseline syscall performance
- **Throughput**: Test maximum syscall rate
- **Latency**: Measure end-to-end syscall latency
- **Memory Usage**: Track memory allocation patterns

## 📚 **References**

### **Standards**
- **Linux System Call Interface**: Based on Linux syscall conventions
- **POSIX.1**: Compliance with POSIX standards where applicable
- **x86_64 ABI**: Follows System V AMD64 ABI

### **Related Documentation**
- **Kernel Internals**: Kernel implementation details
- **User Space Guide**: Application development guide
- **Migration Guide**: Version migration instructions
- **Performance Tuning**: Optimization and tuning guide

---

**Document Version**: 1.0  
**Last Updated**: December 2024  
**Maintainer**: @polymera-os-team  
**Review Cycle**: Monthly  
**Next Review**: January 2025
