//! SQLite 읽기 — 대화를 JSONL 대신 DB에 넣는 에이전트용 (Cursor, VS Code 계열).
//!
//! 크레이트를 쓰지 않고, 파일 포맷을 직접 파싱하지도 않는다. 운영체제가 이미
//! 들고 있는 SQLite 엔진을 실행 시점에 빌려 쓴다 — Windows 는
//! `winsqlite3.dll`(Win10 1803+ 기본 탑재), macOS·Linux 는 `libsqlite3`.
//! WAL·잠금·오버플로 페이지를 우리가 다시 구현하지 않아도 되고 바이너리도
//! 커지지 않는다. 라이브러리가 없으면 그 소스만 조용히 비활성화하고
//! `doctor` 가 이유를 보여준다.
//!
//! **읽기 전용이다.** 남의 편집기 DB를 여는 일이므로 쓰기 플래그를 절대
//! 쓰지 않는다. 편집기가 잡고 있어 열리지 않으면 사본을 떠서 한 번만 다시
//! 시도한다.

use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::OnceLock;

const OK: i32 = 0;
const ROW: i32 = 100;
const DONE: i32 = 101;
const OPEN_READONLY: i32 = 0x0000_0001;
/// 사본을 만들 수 있는 상한. 이보다 큰 DB는 잠겨 있으면 그냥 포기한다.
const MAX_COPY_BYTES: u64 = 256 * 1024 * 1024;

/// 열 이름 → 값. 값은 전부 텍스트로 받는다 — SQLite 가 알아서 변환해 준다.
pub type Row = Vec<(String, String)>;

pub fn get<'a>(row: &'a Row, key: &str) -> Option<&'a str> {
    row.iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
        .filter(|v| !v.is_empty())
}

struct Api {
    open: unsafe extern "C" fn(*const u8, *mut *mut c_void, i32, *const u8) -> i32,
    prepare:
        unsafe extern "C" fn(*mut c_void, *const u8, i32, *mut *mut c_void, *mut *const u8) -> i32,
    step: unsafe extern "C" fn(*mut c_void) -> i32,
    column_count: unsafe extern "C" fn(*mut c_void) -> i32,
    column_name: unsafe extern "C" fn(*mut c_void, i32) -> *const u8,
    column_text: unsafe extern "C" fn(*mut c_void, i32) -> *const u8,
    finalize: unsafe extern "C" fn(*mut c_void) -> i32,
    close: unsafe extern "C" fn(*mut c_void) -> i32,
    errmsg: unsafe extern "C" fn(*mut c_void) -> *const u8,
}

// ------------------------------------------------------------ 라이브러리 로드

#[cfg(windows)]
mod dl {
    use std::ffi::c_void;
    extern "system" {
        fn LoadLibraryA(name: *const u8) -> *mut c_void;
        fn GetProcAddress(handle: *mut c_void, name: *const u8) -> *mut c_void;
    }
    pub const CANDIDATES: &[&str] = &["winsqlite3.dll", "sqlite3.dll"];
    pub unsafe fn open(name: &str) -> *mut c_void {
        LoadLibraryA(super::cstr(name).as_ptr())
    }
    pub unsafe fn symbol(handle: *mut c_void, name: &str) -> *mut c_void {
        GetProcAddress(handle, super::cstr(name).as_ptr())
    }
}

#[cfg(not(windows))]
mod dl {
    use std::ffi::c_void;
    extern "C" {
        fn dlopen(path: *const u8, flags: i32) -> *mut c_void;
        fn dlsym(handle: *mut c_void, name: *const u8) -> *mut c_void;
    }
    const RTLD_LAZY: i32 = 1;
    pub const CANDIDATES: &[&str] = &[
        "libsqlite3.so.0",
        "libsqlite3.dylib",
        "libsqlite3.0.dylib",
        "libsqlite3.so",
    ];
    pub unsafe fn open(name: &str) -> *mut c_void {
        dlopen(super::cstr(name).as_ptr(), RTLD_LAZY)
    }
    pub unsafe fn symbol(handle: *mut c_void, name: &str) -> *mut c_void {
        dlsym(handle, super::cstr(name).as_ptr())
    }
}

fn cstr(s: &str) -> Vec<u8> {
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    v
}

fn load() -> Option<Api> {
    for name in dl::CANDIDATES {
        // 핸들은 프로세스 수명 동안 유지한다. 닫을 이유가 없다.
        let handle = unsafe { dl::open(name) };
        if handle.is_null() {
            continue;
        }
        let sym = |n: &str| {
            let p = unsafe { dl::symbol(handle, n) };
            if p.is_null() {
                None
            } else {
                Some(p)
            }
        };
        // 하나라도 없으면 이 라이브러리는 우리가 아는 SQLite 가 아니다.
        let (open, prepare, step, count, cname, ctext, finalize, close, errmsg) = (
            sym("sqlite3_open_v2")?,
            sym("sqlite3_prepare_v2")?,
            sym("sqlite3_step")?,
            sym("sqlite3_column_count")?,
            sym("sqlite3_column_name")?,
            sym("sqlite3_column_text")?,
            sym("sqlite3_finalize")?,
            sym("sqlite3_close")?,
            sym("sqlite3_errmsg")?,
        );
        return Some(unsafe {
            Api {
                open: std::mem::transmute(open),
                prepare: std::mem::transmute(prepare),
                step: std::mem::transmute(step),
                column_count: std::mem::transmute(count),
                column_name: std::mem::transmute(cname),
                column_text: std::mem::transmute(ctext),
                finalize: std::mem::transmute(finalize),
                close: std::mem::transmute(close),
                errmsg: std::mem::transmute(errmsg),
            }
        });
    }
    None
}

fn api() -> Option<&'static Api> {
    static API: OnceLock<Option<Api>> = OnceLock::new();
    API.get_or_init(load).as_ref()
}

pub fn available() -> bool {
    api().is_some()
}

// ------------------------------------------------------------ 질의

unsafe fn text_at(ptr: *const u8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).into_owned()
}

fn run(api: &Api, file: &Path, sql: &str, flags: i32) -> Result<Vec<Row>, String> {
    let path = cstr(&file.to_string_lossy());
    let statement = cstr(sql);
    let mut db: *mut c_void = null_mut();
    let mut rows = Vec::new();
    unsafe {
        if (api.open)(path.as_ptr(), &mut db, flags, null()) != OK {
            let why = if db.is_null() {
                "열지 못했습니다".to_string()
            } else {
                text_at((api.errmsg)(db))
            };
            (api.close)(db);
            return Err(why);
        }
        let mut stmt: *mut c_void = null_mut();
        if (api.prepare)(db, statement.as_ptr(), -1, &mut stmt, null_mut()) != OK {
            let why = text_at((api.errmsg)(db));
            (api.close)(db);
            return Err(why);
        }
        let columns = (api.column_count)(stmt);
        let names: Vec<String> = (0..columns)
            .map(|i| text_at((api.column_name)(stmt, i)))
            .collect();
        loop {
            let result = (api.step)(stmt);
            if result == DONE {
                break;
            }
            if result != ROW {
                let why = text_at((api.errmsg)(db));
                (api.finalize)(stmt);
                (api.close)(db);
                return Err(why);
            }
            // 값 포인터는 다음 step 까지만 유효하다. 지금 복사한다.
            rows.push(
                names
                    .iter()
                    .enumerate()
                    .map(|(i, name)| (name.clone(), text_at((api.column_text)(stmt, i as i32))))
                    .collect::<Row>(),
            );
        }
        (api.finalize)(stmt);
        (api.close)(db);
    }
    Ok(rows)
}

/// 읽기 전용 질의. 열 이름 그대로 돌려준다.
pub fn query(file: &Path, sql: &str) -> Result<Vec<Row>, String> {
    let api = api().ok_or("SQLite 라이브러리를 찾지 못했습니다")?;
    if !file.exists() {
        return Err("파일이 없습니다".into());
    }
    match run(api, file, sql, OPEN_READONLY) {
        Ok(rows) => Ok(rows),
        // 편집기가 WAL 로 잡고 있으면 읽기 전용 열기도 실패할 수 있다.
        // 원본은 건드리지 않고 사본으로 한 번만 다시 시도한다.
        Err(first) => match copy_aside(file) {
            Some(copy) => run(api, &copy, sql, OPEN_READONLY)
                .map_err(|second| format!("{first} (사본도 실패: {second})")),
            None => Err(first),
        },
    }
}

/// DB 와 WAL·SHM 동반 파일을 임시 폴더로 복사한다. 원본은 읽기만 한다.
fn copy_aside(file: &Path) -> Option<PathBuf> {
    let size = std::fs::metadata(file).ok()?.len();
    if size > MAX_COPY_BYTES {
        return None;
    }
    let mut hash = 0xcbf29ce484222325u64;
    for byte in file.to_string_lossy().as_bytes() {
        hash = (hash ^ *byte as u64).wrapping_mul(0x100000001b3);
    }
    let dir = std::env::temp_dir().join("waid-sqlite");
    std::fs::create_dir_all(&dir).ok()?;
    let base = dir.join(format!("{hash:016x}.db"));
    std::fs::copy(file, &base).ok()?;
    for suffix in ["-wal", "-shm"] {
        let extra = PathBuf::from(format!("{}{suffix}", file.display()));
        let target = PathBuf::from(format!("{}{suffix}", base.display()));
        if extra.exists() {
            let _ = std::fs::copy(&extra, &target);
        } else {
            // 지난 사본의 WAL 이 남아 있으면 낡은 내용을 되살린다.
            let _ = std::fs::remove_file(&target);
        }
    }
    Some(base)
}

#[cfg(test)]
pub fn exec(file: &Path, sql: &str) -> Result<(), String> {
    let api = api().ok_or("SQLite 라이브러리를 찾지 못했습니다")?;
    run(api, file, sql, 0x2 | 0x4).map(|_| ())
}
