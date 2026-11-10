use crate::Category;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    Video,
    Audio,
    Image,
    Archive,
    Document,
    AiModel,
    VirtualDisk,
    Installer,
    Other,
}
struct FileType {
    kind: FileKind,
    label: &'static str,
    extensions: &'static [&'static str],
}
const FILE_TYPES: &[FileType] = &[
    FileType {
        kind: FileKind::Video,
        label: "视频",
        extensions: &["mp4", "mkv", "avi", "mov", "webm"],
    },
    FileType {
        kind: FileKind::Audio,
        label: "音频",
        extensions: &["mp3", "wav", "flac", "aac", "m4a"],
    },
    FileType {
        kind: FileKind::Image,
        label: "图片",
        extensions: &["jpg", "jpeg", "png", "webp", "heic", "psd", "raw"],
    },
    FileType {
        kind: FileKind::Archive,
        label: "压缩包",
        extensions: &["zip", "7z", "rar", "tar", "gz", "iso"],
    },
    FileType {
        kind: FileKind::Document,
        label: "文档",
        extensions: &["pdf", "doc", "docx", "xlsx", "pptx"],
    },
    FileType {
        kind: FileKind::AiModel,
        label: "AI 模型",
        extensions: &["gguf", "safetensors", "onnx", "pt"],
    },
    FileType {
        kind: FileKind::VirtualDisk,
        label: "虚拟磁盘",
        extensions: &["vhd", "vhdx", "vmdk", "qcow2"],
    },
    FileType {
        kind: FileKind::Installer,
        label: "安装包",
        extensions: &["exe", "msi", "msix"],
    },
    FileType {
        kind: FileKind::Other,
        label: "其他",
        extensions: &[],
    },
];
impl FileKind {
    pub fn all() -> impl Iterator<Item = Self> {
        FILE_TYPES.iter().map(|entry| entry.kind)
    }
    pub fn from_extension(extension: &str) -> Self {
        let normalized = extension.to_lowercase();
        FILE_TYPES
            .iter()
            .find(|entry| entry.extensions.contains(&normalized.as_str()))
            .map(|entry| entry.kind)
            .unwrap_or(Self::Other)
    }
    pub fn label(self) -> &'static str {
        FILE_TYPES
            .iter()
            .find(|entry| entry.kind == self)
            .map(|entry| entry.label)
            .unwrap_or("其他")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemKind {
    ApplicationCache,
    DeveloperCache,
    Temporary,
    ProjectArtifact,
    PersonalFile(FileKind),
    InstalledApplication,
}
struct KindInfo {
    category: Category,
    label: &'static str,
}
impl ItemKind {
    fn info(self) -> KindInfo {
        match self {
            Self::ApplicationCache => KindInfo {
                category: Category::Caches,
                label: "缓存",
            },
            Self::DeveloperCache => KindInfo {
                category: Category::Developer,
                label: "缓存",
            },
            Self::Temporary => KindInfo {
                category: Category::Caches,
                label: "临时文件",
            },
            Self::ProjectArtifact => KindInfo {
                category: Category::Developer,
                label: "项目产物",
            },
            Self::PersonalFile(kind) => KindInfo {
                category: Category::LargeFiles,
                label: kind.label(),
            },
            Self::InstalledApplication => KindInfo {
                category: Category::Applications,
                label: "应用",
            },
        }
    }
    pub fn category(self) -> Category {
        self.info().category
    }
    pub fn label(self) -> &'static str {
        self.info().label
    }
    pub fn allows_safe_clean(self) -> bool {
        matches!(
            self,
            Self::ApplicationCache | Self::DeveloperCache | Self::Temporary
        )
    }
}
impl std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}
