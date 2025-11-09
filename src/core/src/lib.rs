use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessErrorKind {
    Parse,
    Encode,
    Decode,
    Unsupported,
    Other,
}

#[derive(Clone, Debug)]
pub struct ProcessError {
    pub kind: ProcessErrorKind,
    pub message: String,
}

pub type ProcessResult = Result<String, ProcessError>;

#[derive(Clone, Debug)]
pub struct Mode {
    pub id: &'static str,
    pub label: &'static str,
}

/// 工具的统一抽象
pub trait Tool: Send + Sync {
    /// 唯一 id（用于路由/持久化）
    fn id(&self) -> &'static str;
    /// 展示名
    fn name(&self) -> &'static str;
    /// 该工具支持的模式列表（顺序即 UI 顺序）
    fn modes(&self) -> &'static [Mode];
    /// 默认模式 id
    fn default_mode(&self) -> &'static str;
    /// 核心处理逻辑：输入 -> 输出
    fn process(&self, mode_id: &str, input: &str) -> ProcessResult;
}

#[derive(Default)]
pub struct Registry {
    inner: Vec<Arc<dyn Tool>>,
}

impl Registry {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.inner.push(tool);
    }
    pub fn tools(&self) -> &[Arc<dyn Tool>] {
        &self.inner
    }
    pub fn by_id(&self, id: &str) -> Option<Arc<dyn Tool>> {
        self.inner.iter().find(|t| t.id() == id).cloned()
    }
}

use once_cell::sync::Lazy;
use std::sync::Mutex;

static REGISTRY: Lazy<Mutex<Registry>> = Lazy::new(|| Mutex::new(Registry::new()));

pub fn register_tool_instance(tool: Arc<dyn Tool>) {
    let mut guard = REGISTRY.lock().unwrap();
    guard.register(tool);
}

pub fn registry() -> impl std::ops::Deref<Target = Registry> {
    struct Guard(std::sync::MutexGuard<'static, Registry>);
    impl std::ops::Deref for Guard {
        type Target = Registry;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    Guard(REGISTRY.lock().unwrap())
}

/// 提供一个安全的“注册”宏，工具 crate 调用它把自己挂到全局
#[macro_export]
macro_rules! register_tool {
    ($ctor:expr) => {
        #[ctor::ctor]
        fn __register_tool__() {
            let tool = std::sync::Arc::new($ctor);
            $crate::register_tool_instance(tool);
        }
    };
}
