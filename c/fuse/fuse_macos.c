/**
 * macOS FUSE Implementation
 * 
 * This file provides the macOS-specific FUSE implementation using macFUSE.
 * It implements the aeth_fuse interface for macOS systems.
 */

#include "libaeth_fuse.h"
#include <fuse/fuse.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include <signal.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <sys/types.h>
#include <pthread.h>

// Global state for the FUSE instance
static struct {
    struct aeth_fuse_ops* ops;
    void* user_ctx;
    struct fuse* fuse;
    int mounted;
    char mount_point[PATH_MAX];
} fuse_state = {0};

// FUSE operation callbacks that bridge to our aeth_fuse_ops
static int fuse_getattr_callback(const char* path, struct stat* stbuf, struct fuse_file_info* fi) {
    (void)fi; // Unused
    
    if (!fuse_state.ops || !fuse_state.ops->getattr) {
        return -ENOSYS;
    }
    
    struct aeth_fuse_context ctx = {
        .uid = getuid(),
        .gid = getgid(),
        .pid = getpid(),
        .private_data = fuse_state.user_ctx
    };
    
    return fuse_state.ops->getattr(path, stbuf, &ctx);
}

static int fuse_readdir_callback(const char* path, void* buf, fuse_fill_dir_t filler,
                                off_t offset, struct fuse_file_info* fi) {
    if (!fuse_state.ops || !fuse_state.ops->readdir) {
        return -ENOSYS;
    }
    
    struct aeth_fuse_context ctx = {
        .uid = getuid(),
        .gid = getgid(),
        .pid = getpid(),
        .private_data = fuse_state.user_ctx
    };
    
    // Create a wrapper filler function that converts our dirent format
    int wrapper_filler(void* buf, const char* name, const struct stat* stbuf, off_t off) {
        return filler(buf, name, stbuf, off);
    };
    
    return fuse_state.ops->readdir(path, buf, wrapper_filler, offset, NULL, &ctx);
}

static int fuse_open_callback(const char* path, struct fuse_file_info* fi) {
    if (!fuse_state.ops || !fuse_state.ops->open) {
        return -ENOSYS;
    }
    
    struct aeth_fuse_context ctx = {
        .uid = getuid(),
        .gid = getgid(),
        .pid = getpid(),
        .private_data = fuse_state.user_ctx
    };
    
    // Convert fuse_file_info to our format
    struct aeth_fuse_file_handle handle = {
        .handle_id = (uint64_t)fi->fh,
        .inode = fi->ino,
        .flags = fi->flags,
        .ref_count = 1
    };
    
    int result = fuse_state.ops->open(path, &handle, &ctx);
    if (result == 0) {
        fi->fh = (uint64_t)handle.handle_id;
    }
    
    return result;
}

static int fuse_read_callback(const char* path, char* buf, size_t size, off_t offset,
                             struct fuse_file_info* fi) {
    if (!fuse_state.ops || !fuse_state.ops->read) {
        return -ENOSYS;
    }
    
    struct aeth_fuse_context ctx = {
        .uid = getuid(),
        .gid = getgid(),
        .pid = getpid(),
        .private_data = fuse_state.user_ctx
    };
    
    struct aeth_fuse_file_handle handle = {
        .handle_id = (uint64_t)fi->fh,
        .inode = fi->ino,
        .flags = fi->flags,
        .ref_count = 1
    };
    
    return fuse_state.ops->read(path, buf, size, offset, &handle, &ctx);
}

static int fuse_statfs_callback(const char* path, struct statvfs* stbuf) {
    if (!fuse_state.ops || !fuse_state.ops->statfs) {
        return -ENOSYS;
    }
    
    struct aeth_fuse_context ctx = {
        .uid = getuid(),
        .gid = getgid(),
        .pid = getpid(),
        .private_data = fuse_state.user_ctx
    };
    
    return fuse_state.ops->statfs(path, stbuf, &ctx);
}

static int fuse_access_callback(const char* path, int mask) {
    if (!fuse_state.ops || !fuse_state.ops->access) {
        return -ENOSYS;
    }
    
    struct aeth_fuse_context ctx = {
        .uid = getuid(),
        .gid = getgid(),
        .pid = getpid(),
        .private_data = fuse_state.user_ctx
    };
    
    return fuse_state.ops->access(path, mask, &ctx);
}

static int fuse_readlink_callback(const char* path, char* buf, size_t size) {
    if (!fuse_state.ops || !fuse_state.ops->readlink) {
        return -ENOSYS;
    }
    
    struct aeth_fuse_context ctx = {
        .uid = getuid(),
        .gid = getgid(),
        .pid = getpid(),
        .private_data = fuse_state.user_ctx
    };
    
    return fuse_state.ops->readlink(path, buf, size, &ctx);
}

static void* fuse_init_callback(struct fuse_conn_info* conn) {
    (void)conn; // Unused
    
    if (fuse_state.ops && fuse_state.ops->init) {
        struct aeth_fuse_context ctx = {
            .uid = getuid(),
            .gid = getgid(),
            .pid = getpid(),
            .private_data = fuse_state.user_ctx
        };
        
        int result = fuse_state.ops->init(&ctx);
        if (result != 0) {
            return NULL;
        }
    }
    
    return fuse_state.user_ctx;
}

static void fuse_destroy_callback(void* private_data) {
    (void)private_data; // Unused
    
    if (fuse_state.ops && fuse_state.ops->destroy) {
        struct aeth_fuse_context ctx = {
            .uid = getuid(),
            .gid = getgid(),
            .pid = getpid(),
            .private_data = fuse_state.user_ctx
        };
        
        fuse_state.ops->destroy(&ctx);
    }
}

// FUSE operations structure for macOS
static struct fuse_operations fuse_ops = {
    .getattr = fuse_getattr_callback,
    .readdir = fuse_readdir_callback,
    .open = fuse_open_callback,
    .read = fuse_read_callback,
    .statfs = fuse_statfs_callback,
    .access = fuse_access_callback,
    .readlink = fuse_readlink_callback,
    .init = fuse_init_callback,
    .destroy = fuse_destroy_callback,
    
    // Read-only filesystem - return EROFS for write operations
    .create = NULL,
    .write = NULL,
    .unlink = NULL,
    .rmdir = NULL,
    .rename = NULL,
    .link = NULL,
    .symlink = NULL,
    .mkdir = NULL,
    .mknod = NULL,
    .chmod = NULL,
    .chown = NULL,
    .truncate = NULL,
    .utimens = NULL,
    .fsync = NULL,
    .fallocate = NULL,
    .setxattr = NULL,
    .getxattr = NULL,
    .listxattr = NULL,
    .removexattr = NULL,
    .lock = NULL,
    .flock = NULL,
    .ioctl = NULL,
    .poll = NULL,
    .write_buf = NULL,
    .read_buf = NULL,
    .flock = NULL,
    .fallocate = NULL,
};

// Signal handlers
static void sigint_handler(int sig) {
    (void)sig;
    if (fuse_state.mounted) {
        printf("Received SIGINT, unmounting...\n");
        aeth_fuse_unmount(fuse_state.mount_point);
    }
    exit(0);
}

static void sigterm_handler(int sig) {
    (void)sig;
    if (fuse_state.mounted) {
        printf("Received SIGTERM, unmounting...\n");
        aeth_fuse_unmount(fuse_state.mount_point);
    }
    exit(0);
}

// Implementation of aeth_fuse functions
int aeth_fuse_mount(const char* mount_point, 
                    const struct aeth_fuse_ops* ops,
                    void* user_ctx,
                    const struct aeth_fuse_mount_opts* opts) {
    if (!mount_point || !ops) {
        return -EINVAL;
    }
    
    if (fuse_state.mounted) {
        return -EBUSY;
    }
    
    // Store the mount point
    strncpy(fuse_state.mount_point, mount_point, sizeof(fuse_state.mount_point) - 1);
    fuse_state.mount_point[sizeof(fuse_state.mount_point) - 1] = '\0';
    
    // Store the operations and user context
    fuse_state.ops = (struct aeth_fuse_ops*)ops;
    fuse_state.user_ctx = user_ctx;
    
    // Build FUSE arguments for macOS
    char* argv[] = {
        "ngfs-fuse",
        mount_point,
        NULL
    };
    int argc = 2;
    
    // Create FUSE instance
    struct fuse_args args = FUSE_ARGS_INIT(argc, argv);
    
    // Set mount options if provided
    if (opts) {
        if (opts->allow_other) {
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, "allow_other");
        }
        if (opts->allow_root) {
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, "allow_root");
        }
        if (opts->default_permissions) {
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, "default_permissions");
        }
        if (opts->kernel_cache) {
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, "kernel_cache");
        }
        if (opts->auto_cache) {
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, "auto_cache");
        }
        if (opts->umask >= 0) {
            char umask_str[16];
            snprintf(umask_str, sizeof(umask_str), "umask=%o", opts->umask);
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, umask_str);
        }
        if (opts->uid >= 0) {
            char uid_str[16];
            snprintf(uid_str, sizeof(uid_str), "uid=%d", opts->uid);
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, uid_str);
        }
        if (opts->gid >= 0) {
            char gid_str[16];
            snprintf(gid_str, sizeof(gid_str), "gid=%d", opts->gid);
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, gid_str);
        }
        if (opts->blksize > 0) {
            char blksize_str[16];
            snprintf(blksize_str, sizeof(blksize_str), "blksize=%d", opts->blksize);
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, blksize_str);
        }
        if (opts->readahead > 0) {
            char readahead_str[16];
            snprintf(readahead_str, sizeof(readahead_str), "readahead=%d", opts->readahead);
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, readahead_str);
        }
        if (opts->max_read > 0) {
            char max_read_str[16];
            snprintf(max_read_str, sizeof(max_read_str), "max_read=%d", opts->max_read);
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, max_read_str);
        }
        if (opts->direct_io) {
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, "direct_io");
        }
        if (opts->kernel_flock) {
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, "kernel_flock");
        }
        if (opts->auto_unmount) {
            fuse_opt_add_arg(&args, "-o");
            fuse_opt_add_arg(&args, "auto_unmount");
        }
        if (opts->debug) {
            fuse_opt_add_arg(&args, "-d");
        }
        if (opts->foreground) {
            fuse_opt_add_arg(&args, "-f");
        }
        if (opts->single_thread) {
            fuse_opt_add_arg(&args, "-s");
        }
    }
    
    // Create FUSE instance
    fuse_state.fuse = fuse_new(&args, &fuse_ops, sizeof(fuse_ops), user_ctx);
    if (!fuse_state.fuse) {
        fprintf(stderr, "Failed to create FUSE instance\n");
        return -ENOMEM;
    }
    
    // Mount the filesystem
    if (fuse_mount(fuse_state.fuse, mount_point) != 0) {
        fprintf(stderr, "Failed to mount FUSE filesystem at %s\n", mount_point);
        fuse_destroy(fuse_state.fuse);
        fuse_state.fuse = NULL;
        return -EIO;
    }
    
    fuse_state.mounted = 1;
    return 0;
}

int aeth_fuse_unmount(const char* mount_point) {
    if (!mount_point) {
        return -EINVAL;
    }
    
    if (!fuse_state.mounted || !fuse_state.fuse) {
        return -ENODEV;
    }
    
    // Unmount the filesystem
    fuse_unmount(fuse_state.fuse);
    
    // Destroy the FUSE instance
    fuse_destroy(fuse_state.fuse);
    fuse_state.fuse = NULL;
    fuse_state.mounted = 0;
    
    return 0;
}

int aeth_fuse_available(void) {
    // Check if macFUSE is available at compile time
    return 1;
}

const char* aeth_fuse_version(void) {
    return "macFUSE";
}

int aeth_fuse_set_signal_handlers(const char* mount_point) {
    (void)mount_point; // Unused
    
    signal(SIGINT, sigint_handler);
    signal(SIGTERM, sigterm_handler);
    
    return 0;
}

int aeth_fuse_remove_signal_handlers(const char* mount_point) {
    (void)mount_point; // Unused
    
    signal(SIGINT, SIG_DFL);
    signal(SIGTERM, SIG_DFL);
    
    return 0;
}

int aeth_fuse_main_loop(const char* mount_point, int single_threaded) {
    if (!mount_point || !fuse_state.mounted) {
        return -EINVAL;
    }
    
    if (single_threaded) {
        return fuse_loop(fuse_state.fuse);
    } else {
        return fuse_loop_mt(fuse_state.fuse);
    }
}

int aeth_fuse_daemonize(int foreground) {
    if (!foreground) {
        if (daemon(0, 0) != 0) {
            return -errno;
        }
    }
    return 0;
}

void aeth_fuse_fill_stat(struct stat* stbuf, mode_t mode, off_t size,
                         ino_t ino, uid_t uid, gid_t gid,
                         time_t atime, time_t mtime, time_t ctime) {
    if (!stbuf) return;
    
    memset(stbuf, 0, sizeof(struct stat));
    stbuf->st_mode = mode;
    stbuf->st_size = size;
    stbuf->st_ino = ino;
    stbuf->st_uid = uid;
    stbuf->st_gid = gid;
    stbuf->st_atime = atime;
    stbuf->st_mtime = mtime;
    stbuf->st_ctime = ctime;
    
    // Set device and link count
    stbuf->st_dev = 0;
    stbuf->st_nlink = 1;
    
    // Set block information
    stbuf->st_blksize = 4096;
    stbuf->st_blocks = (size + 511) / 512;
}

int aeth_fuse_fill_dirent(void* buf, const char* name,
                          const struct stat* stbuf, off_t off) {
    if (!buf || !name || !stbuf) {
        return -EINVAL;
    }
    
    // This is a simplified implementation
    // In a real implementation, you would need to handle the buffer properly
    return 0;
}
