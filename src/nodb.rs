use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DumpPolicy {
    Never,
    Auto,
    #[default]
    OnCall,
    Periodic(Duration),
}

pub struct NoDb {}
