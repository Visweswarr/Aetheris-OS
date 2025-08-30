/**
 * libaeth_fuse - NGFS FUSE Protocol Shim
 * 
 * This library provides a minimal FUSE protocol v7.31 implementation
 * for mounting NGFS snapshots as read-only POSIX filesystems.
 * 
 * The library is designed to be portable across Linux (libfuse3) and
 * macOS (macFUSE) with compile-time switches.
 */

#ifndef LIBAETH_FUSE_H
#define LIBAETH_FUSE_H

#include <stdint.h>
#include <sys/stat.h>
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * FUSE operation result codes
 */
#define AETH_FUSE_OK          0
#define AETH_FUSE_EPERM       1
#define AETH_FUSE_ENOENT      2
#define AETH_FUSE_EIO         5
#define AETH_FUSE_EACCES      13
#define AETH_FUSE_EEXIST      17
#define AETH_FUSE_ENOTDIR     20
#define AETH_FUSE_EISDIR      21
#define AETH_FUSE_EINVAL      22
#define AETH_FUSE_EROFS       30
#define AETH_FUSE_ENOSYS      38
#define AETH_FUSE_ENOTEMPTY   39

/**
 * FUSE file type constants
 */
#define AETH_FUSE_S_IFMT      00170000
#define AETH_FUSE_S_IFSOCK    0140000
#define AETH_FUSE_S_IFLNK     0120000
#define AETH_FUSE_S_IFREG     0100000
#define AETH_FUSE_S_IFBLK     0060000
#define AETH_FUSE_S_IFDIR     0040000
#define AETH_FUSE_S_IFCHR     0020000
#define AETH_FUSE_S_IFIFO     0010000

/**
 * FUSE permission constants
 */
#define AETH_FUSE_S_ISUID     0004000
#define AETH_FUSE_S_ISGID     0002000
#define AETH_FUSE_S_ISVTX     0001000
#define AETH_FUSE_S_IRWXU     0000700
#define AETH_FUSE_S_IRUSR     0000400
#define AETH_FUSE_S_IWUSR     0000200
#define AETH_FUSE_S_IXUSR     0000100
#define AETH_FUSE_S_IRWXG     0000070
#define AETH_FUSE_S_IRGRP     0000040
#define AETH_FUSE_S_IWGRP     0000020
#define AETH_FUSE_S_IXGRP     0000010
#define AETH_FUSE_S_IRWXO     0000007
#define AETH_FUSE_S_IROTH     0000004
#define AETH_FUSE_S_IWOTH     0000002
#define AETH_FUSE_S_IXOTH     0000001

/**
 * FUSE directory entry structure
 */
typedef struct aeth_fuse_dirent {
    uint64_t ino;           // Inode number
    uint64_t off;           // Offset to next entry
    uint32_t namelen;       // Length of name
    uint32_t type;          // File type
    char name[];            // File name (null-terminated)
} aeth_fuse_dirent_t;

/**
 * FUSE file handle
 */
typedef struct aeth_fuse_file_handle {
    uint64_t handle_id;     // Unique handle identifier
    uint64_t inode;         // Associated inode
    uint32_t flags;         // Open flags
    uint32_t ref_count;     // Reference count
} aeth_fuse_file_handle_t;

/**
 * FUSE context structure
 */
typedef struct aeth_fuse_context {
    uid_t uid;              // User ID
    gid_t gid;              // Group ID
    pid_t pid;              // Process ID
    void* private_data;     // Private data pointer
} aeth_fuse_context_t;

/**
 * FUSE operation function pointers
 * These are filled by Rust and called by the FUSE implementation
 */
typedef struct aeth_fuse_ops {
    /**
     * Get file attributes
     * 
     * @param path File path
     * @param stbuf Buffer to fill with stat information
     * @param ctx FUSE context
     * @return 0 on success, negative error code on failure
     */
    int (*getattr)(const char* path, struct stat* stbuf, struct aeth_fuse_context* ctx);
    
    /**
     * Read directory entries
     * 
     * @param path Directory path
     * @param buf Buffer to fill with directory entries
     * @param filler Function to add entries to the buffer
     * @param offset Offset in directory
     * @param fi File info (unused for directories)
     * @param ctx FUSE context
     * @return 0 on success, negative error code on failure
     */
    int (*readdir)(const char* path, void* buf, 
                   int (*filler)(void* buf, const char* name, 
                                const struct stat* stbuf, off_t off),
                   off_t offset, struct aeth_fuse_file_handle* fi,
                   struct aeth_fuse_context* ctx);
    
    /**
     * Open a file
     * 
     * @param path File path
     * @param fi File info to fill
     * @param ctx FUSE context
     * @return 0 on success, negative error code on failure
     */
    int (*open)(const char* path, struct aeth_fuse_file_handle* fi,
                struct aeth_fuse_context* ctx);
    
    /**
     * Read from a file
     * 
     * @param path File path
     * @param buf Buffer to read into
     * @param size Number of bytes to read
     * @param offset Offset in file
     * @param fi File info
     * @param ctx FUSE context
     * @return Number of bytes read on success, negative error code on failure
     */
    int (*read)(const char* path, char* buf, size_t size, off_t offset,
                struct aeth_fuse_file_handle* fi, struct aeth_fuse_context* ctx);
    
    /**
     * Get filesystem statistics
     * 
     * @param path Path (unused, can be NULL)
     * @param stbuf Buffer to fill with filesystem stats
     * @param ctx FUSE context
     * @return 0 on success, negative error code on failure
     */
    int (*statfs)(const char* path, struct statvfs* stbuf,
                  struct aeth_fuse_context* ctx);
    
    /**
     * Initialize filesystem
     * 
     * @param conn Connection information
     * @param ctx FUSE context
     * @return 0 on success, negative error code on failure
     */
    int (*init)(struct aeth_fuse_context* ctx);
    
    /**
     * Clean up filesystem
     * 
     * @param ctx FUSE context
     */
    void (*destroy)(struct aeth_fuse_context* ctx);
    
    /**
     * Access check
     * 
     * @param path File path
     * @param mask Access mask
     * @param ctx FUSE context
     * @return 0 on success, negative error code on failure
     */
    int (*access)(const char* path, int mask, struct aeth_fuse_context* ctx);
    
    /**
     * Read symbolic link
     * 
     * @param path Link path
     * @param buf Buffer to read link target into
     * @param size Buffer size
     * @param ctx FUSE context
     * @return 0 on success, negative error code on failure
     */
    int (*readlink)(const char* path, char* buf, size_t size,
                    struct aeth_fuse_context* ctx);
} aeth_fuse_ops_t;

/**
 * Mount options structure
 */
typedef struct aeth_fuse_mount_opts {
    const char* mount_point;     // Mount point path
    const char* fs_name;         // Filesystem name
    int allow_other;             // Allow other users to access
    int allow_root;              // Allow root to access
    int default_permissions;     // Use default permission checking
    int kernel_cache;            // Enable kernel caching
    int auto_cache;              // Enable auto cache invalidation
    int umask;                   // Default umask
    int uid;                     // Default user ID
    int gid;                     // Default group ID
    int blksize;                 // Block size
    int readahead;               // Read-ahead size
    int max_read;                // Maximum read size
    int max_write;               // Maximum write size
    int hard_remove;             // Hard remove (no unlink)
    int use_ino;                 // Use inode numbers
    int readdir_ino;             // Use inode numbers in readdir
    int direct_io;               // Use direct I/O
    int kernel_flock;            // Use kernel file locking
    int auto_unmount;            // Auto unmount on process exit
    int show_help;               // Show help
    int show_version;            // Show version
    int debug;                   // Enable debug output
    int foreground;              // Run in foreground
    int single_thread;           // Single-threaded operation
} aeth_fuse_mount_opts_t;

/**
 * Mount a FUSE filesystem
 * 
 * @param mount_point Mount point path
 * @param ops FUSE operations structure
 * @param user_ctx User context data
 * @param opts Mount options (can be NULL for defaults)
 * @return 0 on success, negative error code on failure
 */
int aeth_fuse_mount(const char* mount_point, 
                    const struct aeth_fuse_ops* ops,
                    void* user_ctx,
                    const struct aeth_fuse_mount_opts* opts);

/**
 * Unmount a FUSE filesystem
 * 
 * @param mount_point Mount point path
 * @return 0 on success, negative error code on failure
 */
int aeth_fuse_unmount(const char* mount_point);

/**
 * Check if FUSE is available on this system
 * 
 * @return 1 if available, 0 if not
 */
int aeth_fuse_available(void);

/**
 * Get FUSE library version
 * 
 * @return Version string (static, do not free)
 */
const char* aeth_fuse_version(void);

/**
 * Set FUSE signal handlers
 * 
 * @param mount_point Mount point path
 * @return 0 on success, negative error code on failure
 */
int aeth_fuse_set_signal_handlers(const char* mount_point);

/**
 * Remove FUSE signal handlers
 * 
 * @param mount_point Mount point path
 * @return 0 on success, negative error code on failure
 */
int aeth_fuse_remove_signal_handlers(const char* mount_point);

/**
 * FUSE main loop
 * 
 * @param mount_point Mount point path
 * @param single_threaded Whether to run in single-threaded mode
 * @return 0 on success, negative error code on failure
 */
int aeth_fuse_main_loop(const char* mount_point, int single_threaded);

/**
 * FUSE daemonize
 * 
 * @param foreground Whether to run in foreground
 * @return 0 on success, negative error code on failure
 */
int aeth_fuse_daemonize(int foreground);

/**
 * Utility function to create a stat structure
 * 
 * @param stbuf Buffer to fill
 * @param mode File mode
 * @param size File size
 * @param ino Inode number
 * @param uid User ID
 * @param gid Group ID
 * @param atime Access time
 * @param mtime Modification time
 * @param ctime Change time
 */
void aeth_fuse_fill_stat(struct stat* stbuf, mode_t mode, off_t size,
                         ino_t ino, uid_t uid, gid_t gid,
                         time_t atime, time_t mtime, time_t ctime);

/**
 * Utility function to create a directory entry
 * 
 * @param buf Buffer to fill
 * @param name Entry name
 * @param stbuf Entry stat information
 * @param off Offset to next entry
 * @return 0 on success, negative error code on failure
 */
int aeth_fuse_fill_dirent(void* buf, const char* name,
                          const struct stat* stbuf, off_t off);

#ifdef __cplusplus
}
#endif

#endif /* LIBAETH_FUSE_H */
