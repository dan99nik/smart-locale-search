pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB
pub const MAX_TEXT_EXTRACT_BYTES: usize = 2 * 1024 * 1024; // 2 MB
pub const MIN_TEXT_FOR_EMBEDDING: usize = 200; // characters
pub const DB_BATCH_SIZE: usize = 20;
pub const PROGRESS_EMIT_INTERVAL: usize = 10;
pub const MAX_CHUNKS_PER_FILE: usize = 50;

pub const OCR_MAX_IMAGE_SIZE: u64 = 20 * 1024 * 1024; // 20 MB
pub const OCR_MAX_DIMENSION: u32 = 2000; // downscale longest side to this
pub const OCR_MAX_TEXT_CHARS: usize = 10_000; // max stored OCR text per image
pub const OCR_MIN_TEXT_CHARS: usize = 3; // discard results shorter than this

pub const SKIP_EXTENSIONS: &[&str] = &[
    // Executables / binaries
    "exe", "dll", "so", "dylib", "bin", "app", "msi", "deb", "rpm",
    // Archives
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "iso", "dmg", "img",
    // Media (video)
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "m4v", "webm", "mpg", "mpeg",
    // Media (audio)
    "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a", "opus",
    // Compiled / object files
    "o", "obj", "class", "pyc", "pyo", "wasm",
    // Fonts
    "ttf", "otf", "woff", "woff2", "eot",
    // Database files
    "sqlite", "db", "mdb",
    // Disk images / VM
    "vmdk", "vdi", "qcow2", "vhd",
    // Lock / cache
    "lock",
];

pub const SYSTEM_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "__pycache__",
    ".DS_Store",
    ".Trash",
    "Library",
    "$Recycle.Bin",
    "System Volume Information",
    ".cache",
    ".npm",
    ".cargo",
    "dist",
    "build",
    ".next",
    ".nuxt",
    "vendor",
    ".gradle",
    ".m2",
    ".venv",
    "venv",
    ".tox",
    ".eggs",
    "Pods",
    ".svn",
    ".hg",
    "bower_components",
    ".yarn",
    ".pnp",
    "coverage",
    ".terraform",
    ".idea",
    ".vscode",
    "DerivedData",
    "xcuserdata",
    "Caches",
    "CachedData",
    ".Spotlight-V100",
    ".fseventsd",
];

pub const EMBEDDABLE_EXTENSIONS: &[&str] = &[
    // Text
    "txt", "md", "markdown", "rst", "org", "adoc", "textile", "rtf", "log", "nfo",
    // Code
    "js", "ts", "py", "cs", "cpp", "c", "h", "hpp", "json", "html", "htm",
    "css", "scss", "less", "sass", "rs", "go", "java", "rb", "sh", "bash",
    "zsh", "fish", "yaml", "yml", "toml", "xml", "sql", "jsx", "tsx",
    "swift", "kt", "kts", "scala", "clj", "cljs", "ex", "exs", "erl",
    "hs", "lua", "r", "m", "mm", "pl", "pm", "php", "dart", "groovy",
    "vue", "svelte", "astro", "tf", "hcl", "cmake", "make", "makefile",
    "dockerfile", "proto", "graphql", "gql", "ini", "cfg", "conf",
    "properties", "env", "gitignore", "gitattributes", "editorconfig",
    // Documents
    "pdf", "docx", "csv", "xlsx", "xls",
    // Images (OCR text)
    "jpg", "jpeg", "png", "webp",
];

pub fn should_skip_extension(ext: &str) -> bool {
    let lower = ext.to_lowercase();
    SKIP_EXTENSIONS.contains(&lower.as_str())
}

pub fn should_generate_embedding(ext: &str, text_len: usize) -> bool {
    if text_len < MIN_TEXT_FOR_EMBEDDING {
        return false;
    }
    let lower = ext.to_lowercase();
    EMBEDDABLE_EXTENSIONS.contains(&lower.as_str())
}
