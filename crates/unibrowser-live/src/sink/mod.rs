pub mod file;
pub mod session;

pub use file::FileVideoSink;
pub use session::{run_live_session_to_file, LiveSessionReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkKind {
    PreviewWindow,
    RecordFile,
    StreamRtmp,
    StreamSrt,
    StreamWhip,
    VirtualCamera,
    Ndi,
}
