use crate::ids::TargetId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetAccess {
    Read,
    Write,
    ReadWrite,
    Contribute,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetRef {
    pub id: TargetId,
    pub access: TargetAccess,
}

impl TargetRef {
    pub fn read(id: impl Into<String>) -> Self {
        Self {
            id: TargetId(id.into()),
            access: TargetAccess::Read,
        }
    }

    pub fn write(id: impl Into<String>) -> Self {
        Self {
            id: TargetId(id.into()),
            access: TargetAccess::Write,
        }
    }

    pub fn contribute(id: impl Into<String>) -> Self {
        Self {
            id: TargetId(id.into()),
            access: TargetAccess::Contribute,
        }
    }
}
