use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use serde::{Deserialize, Serialize};

include!("modding/model.rs");
include!("modding/resolve.rs");
include!("modding/plugin.rs");

#[cfg(test)]
include!("modding/tests.rs");
