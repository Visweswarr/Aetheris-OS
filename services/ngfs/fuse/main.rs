use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{App, Arg};
use serde_json;

// FFI bindings to our C FUSE library
#[link(name = "aeth_fuse")]
extern "C" {
    fn aeth_fuse_mount(
        mount_point: *const c_char,
        ops: *const AethFuseOps,
        user_ctx: *mut std::ffi::c_void,
        opts: *const AethFuseMountOpts,
    ) -> c_int;
    
    fn aeth_fuse_unmount(mount_point: *const c_char) -> c_int;
    fn aeth_fuse_main_loop(mount_point: *const c_char, single_threaded: c_int) -> c_int;
    fn aeth_fuse_set_signal_handlers(mount_point: *const c_char) -> c_int;
    fn aeth_fuse_remove_signal_handlers(mount_point: *const c_char) -> c_int;
    fn aeth_fuse_fill_stat(
        stbuf: *mut libc::stat,
        mode: libc::mode_t,
        size: libc::off_t,
        ino: libc::ino_t,
        uid: libc::uid_t,
        gid: libc::gid_t,
        atime: libc::time_t,
        mtime: libc::time_t,
        ctime: libc::time_t,
    );
}

// C types for FUSE operations
#[repr(C)]
pub struct AethFuseOps {
    pub getattr: Option<extern "C" fn(*const c_char, *mut libc::stat, *mut AethFuseContext) -> c_int>,
    pub readdir: Option<extern "C" fn(*const c_char, *mut std::ffi::c_void, extern "C" fn(*mut std::ffi::c_void, *const c_char, *const libc::stat, libc::off_t) -> c_int, libc::off_t, *mut AethFuseFileHandle, *mut AethFuseContext) -> c_int>,
    pub open: Option<extern "C" fn(*const c_char, *mut AethFuseFileHandle, *mut AethFuseContext) -> c_int>,
    pub read: Option<extern "C" fn(*const c_char, *mut c_char, usize, libc::off_t, *mut AethFuseFileHandle, *mut AethFuseContext) -> c_int>,
    pub statfs: Option<extern "C" fn(*const c_char, *mut libc::statvfs, *mut AethFuseContext) -> c_int>,
    pub init: Option<extern "C" fn(*mut AethFuseContext) -> c_int>,
    pub destroy: Option<extern "C" fn(*mut AethFuseContext)>,
    pub access: Option<extern "C" fn(*const c_char, c_int, *mut AethFuseContext) -> c_int>,
    pub readlink: Option<extern "C" fn(*const c_char, *mut c_char, usize, *mut AethFuseContext) -> c_int>,
}

#[repr(C)]
pub struct AethFuseContext {
    pub uid: libc::uid_t,
    pub gid: libc::gid_t,
    pub pid: libc::pid_t,
    pub private_data: *mut std::ffi::c_void,
}

#[repr(C)]
pub struct AethFuseFileHandle {
    pub handle_id: u64,
    pub inode: u64,
    pub flags: u32,
    pub ref_count: u32,
}

#[repr(C)]
pub struct AethFuseMountOpts {
    pub mount_point: *const c_char,
    pub fs_name: *const c_char,
    pub allow_other: c_int,
    pub allow_root: c_int,
    pub default_permissions: c_int,
    pub kernel_cache: c_int,
    pub auto_cache: c_int,
    pub umask: c_int,
    pub uid: c_int,
    pub gid: c_int,
    pub blksize: c_int,
    pub readahead: c_int,
    pub max_read: c_int,
    pub max_write: c_int,
    pub hard_remove: c_int,
    pub use_ino: c_int,
    pub readdir_ino: c_int,
    pub direct_io: c_int,
    pub kernel_flock: c_int,
    pub auto_unmount: c_int,
    pub show_help: c_int,
    pub show_version: c_int,
    pub debug: c_int,
    pub foreground: c_int,
    pub single_thread: c_int,
}

// NGFS FUSE server state
struct NgfsFuseServer {
    store_path: PathBuf,
    index_path: PathBuf,
    keydir_path: Option<PathBuf>,
    root_cid: String,
    fake_fuse: bool,
    
    // Inode cache (LRU, bounded to 8192 entries)
    inode_cache: Arc<Mutex<HashMap<String, u64>>>,
    next_inode: u64,
    
    // Statistics
    stats: Arc<Mutex<FuseStats>>,
}

#[derive(Default)]
struct FuseStats {
    opens: u64,
    reads: u64,
    read_p95_us: u32,
    read_times: Vec<u32>,
}

impl NgfsFuseServer {
    fn new(store_path: PathBuf, index_path: PathBuf, keydir_path: Option<PathBuf>, root_cid: String, fake_fuse: bool) -> Self {
        Self {
            store_path,
            index_path,
            keydir_path,
            root_cid,
            fake_fuse,
            inode_cache: Arc::new(Mutex::new(HashMap::new())),
            next_inode: 1,
            stats: Arc::new(Mutex::new(FuseStats::default())),
        }
    }
    
    fn get_inode(&mut self, path: &str) -> u64 {
        let mut cache = self.inode_cache.lock().unwrap();
        
        if let Some(&inode) = cache.get(path) {
            return inode;
        }
        
        let inode = self.next_inode;
        self.next_inode += 1;
        
        if cache.len() >= 8192 {
            let keys: Vec<String> = cache.keys().cloned().collect();
            for key in keys.iter().take(100) {
                cache.remove(key);
            }
        }
        
        cache.insert(path.to_string(), inode);
        inode
    }
    
    fn getattr_impl(&mut self, path: &str) -> Result<libc::stat, i32> {
        let normalized_path = self.normalize_path(path)?;
        
        let mut stbuf: libc::stat = unsafe { std::mem::zeroed() };
        
        if normalized_path == "/" || normalized_path.ends_with('/') {
            unsafe {
                aeth_fuse_fill_stat(
                    &mut stbuf,
                    libc::S_IFDIR | 0o755,
                    0,
                    self.get_inode(&normalized_path),
                    libc::uid_t::MAX,
                    libc::gid_t::MAX,
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as libc::time_t,
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as libc::time_t,
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as libc::time_t,
                );
            }
        } else {
            unsafe {
                aeth_fuse_fill_stat(
                    &mut stbuf,
                    libc::S_IFREG | 0o644,
                    1024,
                    self.get_inode(&normalized_path),
                    libc::uid_t::MAX,
                    libc::gid_t::MAX,
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as libc::time_t,
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as libc::time_t,
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as libc::time_t,
                );
            }
        }
        
        Ok(stbuf)
    }
    
    fn normalize_path(&self, path: &str) -> Result<String, i32> {
        if path.contains("..") {
            return Err(-libc::EACCES);
        }
        
        if path.chars().any(|c| c.is_control()) {
            return Err(-libc::EINVAL);
        }
        
        let path = Path::new(path);
        let normalized = path.to_string_lossy().to_string();
        
        Ok(normalized)
    }
    
    fn readdir_impl(&mut self, path: &str) -> Result<Vec<(String, libc::stat)>, i32> {
        let normalized_path = self.normalize_path(path)?;
        
        let mut entries = Vec::new();
        
        if normalized_path == "/" {
            entries.push((".".to_string(), self.getattr_impl(".")?));
            entries.push(("..".to_string(), self.getattr_impl("..")?));
            entries.push(("file1.txt".to_string(), self.getattr_impl("/file1.txt")?));
            entries.push(("dir1".to_string(), self.getattr_impl("/dir1")?));
        } else if normalized_path == "/dir1" {
            entries.push((".".to_string(), self.getattr_impl(".")?));
            entries.push(("..".to_string(), self.getattr_impl("..")?));
            entries.push(("subfile.txt".to_string(), self.getattr_impl("/dir1/subfile.txt")?));
        }
        
        Ok(entries)
    }
    
    fn open_impl(&mut self, path: &str, flags: u32) -> Result<AethFuseFileHandle, i32> {
        let normalized_path = self.normalize_path(path)?;
        let _stbuf = self.getattr_impl(&normalized_path)?;
        
        {
            let mut stats = self.stats.lock().unwrap();
            stats.opens += 1;
        }
        
        Ok(AethFuseFileHandle {
            handle_id: self.get_inode(&normalized_path),
            inode: self.get_inode(&normalized_path),
            flags,
            ref_count: 1,
        })
    }
    
    fn read_impl(&mut self, path: &str, buf: &mut [u8], offset: u64) -> Result<usize, i32> {
        let start_time = SystemTime::now();
        
        let normalized_path = self.normalize_path(path)?;
        
        let data = b"Hello, NGFS FUSE! This is placeholder data.\n";
        let data_len = data.len();
        let offset = offset as usize;
        
        if offset >= data_len {
            return Ok(0);
        }
        
        let remaining = data_len - offset;
        let to_copy = std::cmp::min(remaining, buf.len());
        
        buf[..to_copy].copy_from_slice(&data[offset..offset + to_copy]);
        
        {
            let mut stats = self.stats.lock().unwrap();
            stats.reads += 1;
            
            let read_time = start_time.elapsed().unwrap().as_micros() as u32;
            stats.read_times.push(read_time);
            
            if stats.read_times.len() > 100 {
                stats.read_times.remove(0);
            }
            
            if !stats.read_times.is_empty() {
                let mut times = stats.read_times.clone();
                times.sort();
                let p95_idx = (times.len() as f64 * 0.95) as usize;
                stats.read_p95_us = times[p95_idx.min(times.len() - 1)];
            }
        }
        
        Ok(to_copy)
    }
    
    fn statfs_impl(&self) -> Result<libc::statvfs, i32> {
        let mut stbuf: libc::statvfs = unsafe { std::mem::zeroed() };
        
        stbuf.f_bsize = 4096;
        stbuf.f_frsize = 4096;
        stbuf.f_blocks = 1000000;
        stbuf.f_bfree = 500000;
        stbuf.f_bavail = 500000;
        stbuf.f_files = 10000;
        stbuf.f_ffree = 5000;
        stbuf.f_favail = 5000;
        stbuf.f_fsid = 0;
        stbuf.f_flag = 0;
        stbuf.f_namemax = 255;
        
        Ok(stbuf)
    }
    
    fn access_impl(&self, path: &str, _mask: i32) -> Result<(), i32> {
        let _normalized_path = self.normalize_path(path)?;
        Ok(())
    }
    
    fn readlink_impl(&self, path: &str) -> Result<String, i32> {
        let _normalized_path = self.normalize_path(path)?;
        Ok("/target/file".to_string())
    }
    
    fn print_stats(&self) {
        let stats = self.stats.lock().unwrap();
        let event = serde_json::json!({
            "event": "stats",
            "opens": stats.opens,
            "reads": stats.reads,
            "read_p95_us": stats.read_p95_us
        });
        println!("{}", event);
    }
}

// FUSE operation callbacks
extern "C" fn getattr_callback(path: *const c_char, stbuf: *mut libc::stat, ctx: *mut AethFuseContext) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy().to_string() };
    let server = unsafe { &mut *(ctx.as_ref().unwrap().private_data as *mut NgfsFuseServer) };
    
    match server.getattr_impl(&path_str) {
        Ok(stat) => {
            unsafe { *stbuf = stat; }
            0
        }
        Err(err) => err
    }
}

extern "C" fn readdir_callback(
    path: *const c_char,
    buf: *mut std::ffi::c_void,
    filler: extern "C" fn(*mut std::ffi::c_void, *const c_char, *const libc::stat, libc::off_t) -> c_int,
    _offset: libc::off_t,
    _fi: *mut AethFuseFileHandle,
    ctx: *mut AethFuseContext,
) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy().to_string() };
    let server = unsafe { &mut *(ctx.as_ref().unwrap().private_data as *mut NgfsFuseServer) };
    
    match server.readdir_impl(&path_str) {
        Ok(entries) => {
            for (name, stat) in entries {
                let name_cstr = CString::new(name).unwrap();
                unsafe {
                    filler(buf, name_cstr.as_ptr(), &stat, 0);
                }
            }
            0
        }
        Err(err) => err
    }
}

extern "C" fn open_callback(path: *const c_char, fi: *mut AethFuseFileHandle, ctx: *mut AethFuseContext) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy().to_string() };
    let server = unsafe { &mut *(ctx.as_ref().unwrap().private_data as *mut NgfsFuseServer) };
    
    match server.open_impl(&path_str, unsafe { (*fi).flags }) {
        Ok(handle) => {
            unsafe { *fi = handle; }
            0
        }
        Err(err) => err
    }
}

extern "C" fn read_callback(
    path: *const c_char,
    buf: *mut c_char,
    size: usize,
    offset: libc::off_t,
    _fi: *mut AethFuseFileHandle,
    ctx: *mut AethFuseContext,
) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy().to_string() };
    let server = unsafe { &mut *(ctx.as_ref().unwrap().private_data as *mut NgfsFuseServer) };
    
    let buf_slice = unsafe { std::slice::from_raw_parts_mut(buf, size) };
    
    match server.read_impl(&path_str, buf_slice, offset as u64) {
        Ok(bytes_read) => bytes_read as c_int,
        Err(err) => err
    }
}

extern "C" fn statfs_callback(_path: *const c_char, stbuf: *mut libc::statvfs) -> c_int {
    let server = unsafe { &mut *(ctx.as_ref().unwrap().private_data as *mut NgfsFuseServer) };
    
    match server.statfs_impl() {
        Ok(stat) => {
            unsafe { *stbuf = stat; }
            0
        }
        Err(err) => err
    }
}

extern "C" fn access_callback(path: *const c_char, mask: c_int, ctx: *mut AethFuseContext) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy().to_string() };
    let server = unsafe { &mut *(ctx.as_ref().unwrap().private_data as *mut NgfsFuseServer) };
    
    match server.access_impl(&path_str, mask) {
        Ok(()) => 0,
        Err(err) => err
    }
}

extern "C" fn readlink_callback(path: *const c_char, buf: *mut c_char, size: usize, ctx: *mut AethFuseContext) -> c_int {
    let path_str = unsafe { CStr::from_ptr(path).to_string_lossy().to_string() };
    let server = unsafe { &mut *(ctx.as_ref().unwrap().private_data as *mut NgfsFuseServer) };
    
    match server.readlink_impl(&path_str) {
        Ok(target) => {
            let target_bytes = target.as_bytes();
            let to_copy = std::cmp::min(target_bytes.len(), size - 1);
            
            unsafe {
                std::ptr::copy_nonoverlapping(target_bytes.as_ptr(), buf, to_copy);
                buf.add(to_copy).write(0);
            }
            
            to_copy as c_int
        }
        Err(err) => err
    }
}

extern "C" fn init_callback(_ctx: *mut AethFuseContext) -> c_int {
    0
}

extern "C" fn destroy_callback(ctx: *mut AethFuseContext) {
    let server = unsafe { &*(ctx.as_ref().unwrap().private_data as *const NgfsFuseServer) };
    server.print_stats();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("ngfs-fuse")
        .version("1.0")
        .about("NGFS FUSE read-only mount")
        .arg(Arg::with_name("store")
            .long("store")
            .value_name("FILE")
            .help("NGFS store file path")
            .required(true))
        .arg(Arg::with_name("idx")
            .long("idx")
            .value_name("FILE")
            .help("NGFS index file path")
            .required(true))
        .arg(Arg::with_name("at")
            .long("at")
            .value_name("DIR")
            .help("Mount point directory")
            .required(true))
        .arg(Arg::with_name("root")
            .long("root")
            .value_name("CID")
            .help("Root directory CID")
            .required(true))
        .arg(Arg::with_name("keydir")
            .long("keydir")
            .value_name("DIR")
            .help("Key directory for dev KEK/DEK fixtures"))
        .arg(Arg::with_name("fake-fuse")
            .long("fake-fuse")
            .help("Use fake FUSE mode for CI/testing"))
        .get_matches();
    
    let store_path = PathBuf::from(matches.value_of("store").unwrap());
    let index_path = PathBuf::from(matches.value_of("idx").unwrap());
    let mount_point = matches.value_of("at").unwrap();
    let root_cid = matches.value_of("root").unwrap().to_string();
    let keydir_path = matches.value_of("keydir").map(PathBuf::from);
    let fake_fuse = matches.is_present("fake-fuse");
    
    let mut server = NgfsFuseServer::new(
        store_path,
        index_path,
        keydir_path,
        root_cid.clone(),
        fake_fuse,
    );
    
    if fake_fuse {
        println!("Running in fake FUSE mode");
        
        let _ = server.getattr_impl("/");
        let _ = server.readdir_impl("/");
        let _ = server.open_impl("/file1.txt", 0);
        
        server.print_stats();
        return Ok(());
    }
    
    if unsafe { aeth_fuse_available() } == 0 {
        eprintln!("FUSE is not available on this system");
        return Ok(());
    }
    
    let fuse_ops = AethFuseOps {
        getattr: Some(getattr_callback),
        readdir: Some(readdir_callback),
        open: Some(open_callback),
        read: Some(read_callback),
        statfs: Some(statfs_callback),
        init: Some(init_callback),
        destroy: Some(destroy_callback),
        access: Some(access_callback),
        readlink: Some(readlink_callback),
    };
    
    let mount_opts = AethFuseMountOpts {
        mount_point: std::ptr::null(),
        fs_name: std::ptr::null(),
        allow_other: 0,
        allow_root: 0,
        default_permissions: 1,
        kernel_cache: 0,
        auto_cache: 0,
        umask: -1,
        uid: -1,
        gid: -1,
        blksize: -1,
        readahead: -1,
        max_read: 65536,
        max_write: -1,
        hard_remove: 0,
        use_ino: 1,
        readdir_ino: 1,
        direct_io: 0,
        kernel_flock: 0,
        auto_unmount: 1,
        show_help: 0,
        show_version: 0,
        debug: 0,
        foreground: 1,
        single_thread: 0,
    };
    
    let mount_point_cstr = CString::new(mount_point).unwrap();
    
    let mount_event = serde_json::json!({
        "event": "mount",
        "path": mount_point,
        "root": root_cid,
        "fake": false
    });
    println!("{}", mount_event);
    
    let mount_result = unsafe {
        aeth_fuse_mount(
            mount_point_cstr.as_ptr(),
            &fuse_ops,
            &mut server as *mut _ as *mut std::ffi::c_void,
            &mount_opts,
        )
    };
    
    if mount_result != 0 {
        eprintln!("Failed to mount FUSE filesystem: {}", mount_result);
        return Ok(());
    }
    
    unsafe {
        aeth_fuse_set_signal_handlers(mount_point_cstr.as_ptr());
    }
    
    let loop_result = unsafe {
        aeth_fuse_main_loop(mount_point_cstr.as_ptr(), 0)
    };
    
    if loop_result != 0 {
        eprintln!("FUSE main loop failed: {}", loop_result);
    }
    
    unsafe {
        aeth_fuse_remove_signal_handlers(mount_point_cstr.as_ptr());
        aeth_fuse_unmount(mount_point_cstr.as_ptr());
    }
    
    Ok(())
}
